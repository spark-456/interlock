//! `app.gmail.reply` — Reply (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use super::common;
use il_sdk::prelude::*;

/// Reply (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailReply;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.gmail.reply", Category::App, "Reply (Gmail)", "Replies in the original email thread.")
        .icon("mail")
        .app("gmail")
        .external()
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
            "required": [
                "threadId",
                "body"
            ],
            "properties": {
                "threadId": {
                    "type": "string",
                    "title": "Thread",
                    "default": "{{ item.threadId }}",
                    "x-widget": "expression"
                },
                "body": {
                    "type": "string",
                    "title": "Reply",
                    "x-widget": "longtext"
                },
                "cc": {
                    "type": "string",
                    "title": "Cc"
                }
            }
        }))
        .example(
            "Send draft",
            json!({
                "threadId": "{{ item.threadId }}",
                "body": "{{ item.reply }}"
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppGmailReply {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let thread_id =
            common::id(ctx.settings["threadId"].as_str().ok_or_else(|| NodeError::Failed("missing threadId".into()))?)?;
        let body = ctx.settings["body"].as_str().ok_or_else(|| NodeError::Failed("missing body".into()))?;
        let url = common::endpoint(ctx, &format!("threads/{thread_id}?format=metadata"));
        let token = common::token(ctx).await?;
        let thread: Value = ctx
            .http()
            .get(&url)
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
        let headers = thread["messages"]
            .as_array()
            .and_then(|messages| messages.last())
            .and_then(|message| message["payload"]["headers"].as_array())
            .ok_or_else(|| NodeError::Failed("Gmail thread has no messages".into()))?;
        let value = |name: &str| {
            headers
                .iter()
                .find(|header| header["name"].as_str().is_some_and(|v| v.eq_ignore_ascii_case(name)))
                .and_then(|header| header["value"].as_str())
                .unwrap_or("")
        };
        let to = value("From");
        let subject = value("Subject");
        let message_id = value("Message-ID");
        if to.is_empty() || message_id.is_empty() {
            return Err(NodeError::Failed("Gmail thread lacks reply headers".into()));
        }
        let subject =
            if subject.to_ascii_lowercase().starts_with("re:") { subject.into() } else { format!("Re: {subject}") };
        let mut extra = vec![("In-Reply-To", message_id), ("References", message_id)];
        if let Some(cc) = ctx.settings["cc"].as_str() {
            extra.push(("Cc", cc));
        }
        let raw = common::raw(to, &subject, body, &extra)?;
        let result = common::send(ctx, raw, Some(thread_id)).await?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["gmail"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailReply)) }

#[cfg(test)]
mod tests {
    use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
    use il_sdk::{prelude::*, services::Credential, testkit::NodeHarness};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailReply);
    }

    #[tokio::test]
    async fn c01_gmail_reply_uses_thread_headers() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gmail/v1/users/me/threads/thread1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"messages":[{"payload":{"headers":[
                {"name":"From","value":"customer@example.com"},{"name":"Subject","value":"Question"},
                {"name":"Message-ID","value":"<msg@example.com>"}]}}]})))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/gmail/v1/users/me/messages/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"reply1"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppGmailReply)
            .settings(json!({"threadId":"thread1","body":"Thanks","_base":server.uri()}))
            .credential("google", "cred_1");
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["gmail"]["id"], "reply1");
        let requests = server.received_requests().await.unwrap();
        let send: Value = serde_json::from_slice(&requests[1].body).unwrap();
        assert_eq!(send["threadId"], "thread1");
        let mime = URL_SAFE_NO_PAD.decode(send["raw"].as_str().unwrap()).unwrap();
        assert!(String::from_utf8_lossy(&mime).contains("In-Reply-To: <msg@example.com>"));
    }
}
