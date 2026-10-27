//! `app.gmail.send` — Send email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use super::common;
use il_sdk::prelude::*;

/// Send email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.gmail.send",
        Category::App,
        "Send email (Gmail)",
        "Sends a new email. Recipients come from settings, not from item data, unless you map them.",
    )
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
            "to",
            "subject",
            "body"
        ],
        "properties": {
            "to": {
                "type": "string",
                "title": "To"
            },
            "subject": {
                "type": "string",
                "title": "Subject"
            },
            "body": {
                "type": "string",
                "title": "Body",
                "x-widget": "longtext"
            }
        }
    }))
    .example(
        "Weekly report",
        json!({
            "to": "team@larkspur.example",
            "subject": "Weekly promos",
            "body": "{{ item.text }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppGmailSend {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let to = ctx.settings["to"].as_str().ok_or_else(|| NodeError::Failed("missing to".into()))?;
        let subject = ctx.settings["subject"].as_str().ok_or_else(|| NodeError::Failed("missing subject".into()))?;
        let body = ctx.settings["body"].as_str().ok_or_else(|| NodeError::Failed("missing body".into()))?;
        let raw = common::raw(to, subject, body, &[])?;
        let result = common::send(ctx, raw, None).await?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["gmail"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailSend)) }

#[cfg(test)]
mod tests {
    use il_sdk::prelude::*;
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailSend);
    }

    #[tokio::test]
    async fn c01_gmail_send_uses_bound_credential_and_dedupes() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/gmail/v1/users/me/messages/send"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"msg_1"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppGmailSend)
            .settings(json!({"to":"team@example.com","subject":"Hello","body":"Report","_base":server.uri()}))
            .credential("google", "cred_1");
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["gmail"]["id"], "msg_1");
        harness.run().await.unwrap();
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
