//! `app.slack.send` — Send message (Slack).

use il_sdk::prelude::*;

/// Send message (Slack) node.
#[derive(Debug, Default)]
pub struct AppSlackSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.slack.send", Category::App, "Send message (Slack)", "Posts a message to a Slack channel.")
        .icon("message")
        .status(NodeStatus::Ready)
        .app("slack")
        .external()
        .credential(cred("slack", "slack", CredentialKind::OAuth2, "Slack workspace", &["chat:write"]))
        .settings_schema(json!({
            "type": "object",
            "required": [
                "channel",
                "text"
            ],
            "properties": {
                "channel": {
                    "type": "string",
                    "title": "Channel"
                },
                "text": {
                    "type": "string",
                    "title": "Message",
                    "x-widget": "longtext"
                }
            }
        }))
        .example(
            "Notify",
            json!({
                "channel": "#marketing",
                "text": "New promos are ready."
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppSlackSend {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let channel =
            ctx.settings["channel"].as_str().ok_or_else(|| NodeError::Failed("channel is required".into()))?;
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("text is required".into()))?;
        let credential = ctx.credential("slack").await?;
        let token = credential.secret["access_token"]
            .as_str()
            .or_else(|| credential.secret["bot_token"].as_str())
            .ok_or_else(|| NodeError::Failed("Slack access token is missing".into()))?
            .to_owned();
        let base = super::api_base(&ctx.settings);
        super::post_message(ctx.http(), &base, &token, channel, text, None).await?;
        Ok(NodeOutput::out(input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSlackSend)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSlackSend);
    }

    #[tokio::test]
    async fn c02_slack_send_checks_logical_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/chat.postMessage"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": false, "error": "channel_not_found"})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppSlackSend)
            .settings(json!({"channel":"C123","text":"hi","_base":server.uri()}))
            .credential("slack", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "slack".into(),
            secret: json!({"access_token":"test-token"}),
            meta: json!({}),
        });
        let err = harness.run().await.expect_err("Slack logical error must fail the node");
        assert!(err.to_string().contains("channel_not_found"));
    }
}
