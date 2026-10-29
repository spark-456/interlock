//! `app.discord.send` — Send message (Discord).
//!
//! Credentials supply a webhook URL or a bot token and channel id.

use il_sdk::prelude::*;

/// Send message (Discord) node.
#[derive(Debug, Default)]
pub struct AppDiscordSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.discord.send",
        Category::App,
        "Send message (Discord)",
        "Posts a message with optional attachments to a Discord channel via webhook.",
    )
    .icon("message")
    .status(NodeStatus::Ready)
    .app("discord")
    .external()
    .credential(cred("discord", "discord", CredentialKind::Webhook, "Discord webhook", &[]))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "content"
        ],
        "properties": {
            "content": {
                "type": "string",
                "title": "Message",
                "x-widget": "longtext"
            },
            "attachments": {
                "type": "string",
                "title": "Attachments",
                "description": "File references to attach.",
                "x-widget": "expression"
            },
            "channelId": {
                "type": "string",
                "title": "Bot channel ID",
                "description": "Required when using a Discord bot token instead of a webhook."
            }
        }
    }))
    .example(
        "Post clip",
        json!({
            "content": "{{ item.post }}",
            "attachments": "{{ item.files.clip_16x9 }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppDiscordSend {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let content =
            ctx.settings["content"].as_str().ok_or_else(|| NodeError::Failed("content is required".into()))?;
        let credential = ctx.credential("discord").await?;
        let (url, bot_token) = if let Some(url) = credential.secret["url"].as_str() {
            (url.to_owned(), None)
        } else if let Some(token) = credential.secret["bot_token"].as_str() {
            let channel = ctx.settings["channelId"]
                .as_str()
                .ok_or_else(|| NodeError::Failed("Discord bot channel ID is required".into()))?;
            if !channel.chars().all(|c| c.is_ascii_digit()) {
                return Err(NodeError::Failed("invalid Discord channel ID".into()));
            }
            let base = "https://discord.com";
            #[cfg(test)]
            let base = ctx.settings["_base"].as_str().unwrap_or(base);
            (format!("{base}/api/v10/channels/{channel}/messages"), Some(token))
        } else {
            return Err(NodeError::Failed("Discord webhook URL or bot token missing".into()));
        };
        // Once per run and item, so a retry or a resumed run never posts twice.
        let send = super::send(ctx, &url, bot_token, content, &ctx.settings["attachments"]);
        crate::rest::once(ctx, "discord:send", async { send.await.map(Value::from) }).await?;
        Ok(NodeOutput::out(input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppDiscordSend)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::{Credential, FileStore};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppDiscordSend);
    }

    #[tokio::test]
    async fn c02_discord_webhook_attaches_file() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/webhooks/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"message-1"})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppDiscordSend).credential("discord", "cred");
        let file = harness.fakes.files.put("note.txt", "text/plain", "hello".into()).await.unwrap();
        let harness = harness.settings(json!({"content":"hello","attachments":file.id}));
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "discord".into(),
            secret: json!({"url":format!("{}/api/webhooks/test",server.uri())}),
            meta: json!({}),
        });
        assert!(harness.run().await.is_ok());
        let requests = server.received_requests().await.unwrap();
        let body = String::from_utf8_lossy(&requests[0].body);
        assert!(body.contains("note.txt") && body.contains("hello"));
    }

    #[tokio::test]
    async fn c02_discord_bot_uses_channel_and_authorization() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v10/channels/123/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id":"message-2"})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppDiscordSend)
            .settings(json!({"content":"hi","channelId":"123","_base":server.uri()}))
            .credential("discord", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "discord".into(),
            secret: json!({"bot_token":"secret"}),
            meta: json!({}),
        });
        assert!(harness.run().await.is_ok());
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].headers["authorization"].to_str().unwrap(), "Bot secret");
    }
}
