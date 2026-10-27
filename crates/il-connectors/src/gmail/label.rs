//! `app.gmail.label` — Label email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use super::common;
use il_sdk::prelude::*;

/// Label email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailLabel;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.gmail.label", Category::App, "Label email (Gmail)", "Adds or removes labels on an email.")
        .icon("tag")
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
                "messageId"
            ],
            "properties": {
                "messageId": {
                    "type": "string",
                    "title": "Message",
                    "default": "{{ item.messageId }}",
                    "x-widget": "expression"
                },
                "addLabels": {
                    "type": "array",
                    "title": "Add labels",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "removeLabels": {
                    "type": "array",
                    "title": "Remove labels",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                }
            }
        }))
        .example(
            "Mark spam",
            json!({
                "messageId": "{{ item.messageId }}",
                "addLabels": [
                    "SPAM"
                ]
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppGmailLabel {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let message_id = common::id(
            ctx.settings["messageId"].as_str().ok_or_else(|| NodeError::Failed("missing messageId".into()))?,
        )?;
        let key = format!("gmail:label:{}", ctx.dedupe_key());
        if let Some(saved) = ctx.services.store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
            let mut item = input.main().first().cloned().unwrap_or_default();
            item.json["gmail"] = saved;
            return Ok(NodeOutput::out(vec![item]));
        }
        let url = common::endpoint(ctx, &format!("messages/{message_id}/modify"));
        let token = common::token(ctx).await?;
        let response = ctx
            .http()
            .post(&url)
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .json(&json!({"addLabelIds":ctx.settings["addLabels"].as_array().cloned().unwrap_or_default(),
                "removeLabelIds":ctx.settings["removeLabels"].as_array().cloned().unwrap_or_default()}))
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let result: Value = response.json().await.map_err(|e| NodeError::Failed(e.to_string()))?;
        ctx.services.store.kv_put(&key, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["gmail"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailLabel)) }

#[cfg(test)]
mod tests {
    use il_sdk::{prelude::*, services::Credential, testkit::NodeHarness};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailLabel);
    }

    #[tokio::test]
    async fn c01_gmail_label_sends_modify_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gmail/v1/users/me/messages/msg1/modify"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"msg1","labelIds":["SPAM"]})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppGmailLabel)
            .settings(json!({"messageId":"msg1","addLabels":["SPAM"],"_base":server.uri()}))
            .credential("google", "cred_1");
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["gmail"]["id"], "msg1");
        harness.run().await.unwrap();
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
