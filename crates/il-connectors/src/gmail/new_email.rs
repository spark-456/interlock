//! `app.gmail.new_email` — New email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;
use il_sdk::{
    TriggerCtx, TriggerHandle,
    poll::{Poller, spawn_polling},
};
use std::{sync::Arc, time::Duration};

/// New email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailNewEmail;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.gmail.new_email",
        Category::Trigger,
        "New email (Gmail)",
        "Starts the flow for each new email matching a label or search.",
    )
    .icon("mail")
    .app("gmail")
    .trigger()
    .status(NodeStatus::Ready)
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/gmail.modify"],
    ))
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "label": {
                "type": "string",
                "title": "Label",
                "default": "INBOX"
            },
            "query": {
                "type": "string",
                "title": "Search",
                "description": "Gmail search, e.g. `to:support@larkspur.example`."
            },
            "pollMinutes": {
                "type": "integer",
                "title": "Check every (min)",
                "default": 1,
                "minimum": 1,
                "maximum": 60
            }
        }
    }))
    .example(
        "Support inbox",
        json!({
            "label": "INBOX",
            "query": "to:support@larkspur.example"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppGmailNewEmail {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for AppGmailNewEmail {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        let minutes = ctx.settings["pollMinutes"].as_u64().unwrap_or(1).clamp(1, 60);
        Ok(spawn_polling(ctx, "app.gmail.new_email", Duration::from_secs(minutes * 60), Arc::new(GmailPoller)))
    }
}

struct GmailPoller;

#[async_trait]
impl Poller for GmailPoller {
    async fn poll(&self, ctx: &TriggerCtx, cursor: Option<Value>) -> Result<(Vec<Item>, Option<Value>), NodeError> {
        let token = crate::oauth::google_token(&ctx.services, &ctx.workspace_id, &ctx.credential_ids).await?;
        let token = token.as_str();
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://gmail.googleapis.com");
        #[cfg(not(test))]
        let base = "https://gmail.googleapis.com";
        let mut url = url::Url::parse(&format!("{base}/gmail/v1/users/me/messages"))
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        url.query_pairs_mut()
            .append_pair("maxResults", "100")
            .append_pair("labelIds", ctx.settings["label"].as_str().unwrap_or("INBOX"));
        if let Some(query) = ctx.settings["query"].as_str() {
            url.query_pairs_mut().append_pair("q", query);
        }
        let response: Value = ctx
            .services
            .http
            .get(url.as_str())
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .json()
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let messages = response["messages"].as_array().cloned().unwrap_or_default();
        let newest = messages.first().and_then(|message| message["id"].as_str()).map(str::to_owned);
        let previous = cursor.as_ref().and_then(Value::as_str);
        if previous.is_none() {
            return Ok((Vec::new(), newest.map(Value::String)));
        }
        let mut unseen = Vec::new();
        for message in &messages {
            if message["id"].as_str() == previous {
                break;
            }
            if let Some(id) = message["id"].as_str() {
                unseen.push(id.to_owned());
            }
        }
        let mut items = Vec::new();
        for id in unseen.into_iter().rev() {
            if !id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
                continue;
            }
            let detail: Value = ctx
                .services
                .http
                .get(&format!("{base}/gmail/v1/users/me/messages/{id}?format=full"))
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| NodeError::Retryable(e.to_string()))?
                .error_for_status()
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .json()
                .await
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            items.push(Item::json(json!({"messageId":id,"threadId":detail["threadId"],
                "snippet":detail["snippet"],"payload":detail["payload"]})));
        }
        Ok((items, newest.map(Value::String)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailNewEmail)) }

#[cfg(test)]
mod tests {
    use il_sdk::{
        TriggerCtx,
        poll::Poller,
        services::Credential,
        testkit::{FakeRunner, FakeServices},
    };
    use serde_json::json;
    use std::{collections::BTreeMap, sync::Arc};
    use tokio_util::sync::CancellationToken;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailNewEmail);
    }

    #[tokio::test]
    async fn c01_gmail_poller_uses_cursor() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"messages":[{"id":"msg2"},{"id":"msg1"}]})))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/messages/msg2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"threadId":"thread2","snippet":"Hello"})))
            .mount(&server)
            .await;
        let fakes = FakeServices::default();
        fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        let ctx = TriggerCtx {
            flow_id: "flow".into(),
            workspace_id: "ws".into(),
            node_id: "n".into(),
            settings: json!({"_base":server.uri()}),
            credential_ids: BTreeMap::from([("google".into(), "cred".into())]),
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            cancel: CancellationToken::new(),
        };
        let (items, cursor) = super::GmailPoller.poll(&ctx, None).await.unwrap();
        assert!(items.is_empty());
        assert_eq!(cursor, Some(json!("msg2")));
        let (items, cursor) = super::GmailPoller.poll(&ctx, Some(json!("msg1"))).await.unwrap();
        assert_eq!(items[0].json["messageId"], "msg2");
        assert_eq!(cursor, Some(json!("msg2")));
    }
}
