//! `app.telegram.send` — Send message (Telegram).
//!
//! Bot token is obtained from `NodeCtx`, never from item data.

use il_sdk::prelude::*;

/// Send message (Telegram) node.
#[derive(Debug, Default)]
pub struct AppTelegramSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.telegram.send",
        Category::App,
        "Send message (Telegram)",
        "Sends a message or media to a Telegram chat via your bot.",
    )
    .icon("send")
    .status(NodeStatus::Ready)
    .app("telegram")
    .external()
    .credential(cred("telegram", "telegram", CredentialKind::BotToken, "Telegram bot token", &[]))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "chatId",
            "text"
        ],
        "properties": {
            "chatId": {
                "type": "string",
                "title": "Chat id"
            },
            "text": {
                "type": "string",
                "title": "Message",
                "x-widget": "longtext"
            },
            "media": {
                "type": "string",
                "title": "Media",
                "x-widget": "expression"
            }
        }
    }))
    .example(
        "Notify channel",
        json!({
            "chatId": "@larkspur",
            "text": "{{ item.post }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppTelegramSend {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let chat_id = ctx.settings["chatId"].as_str().ok_or_else(|| NodeError::Failed("chatId is required".into()))?;
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("text is required".into()))?;
        let token = ctx.credential("telegram").await?.secret["token"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("Telegram bot token is missing".into()))?
            .to_owned();
        // Once per run and item, so a retry or a resumed run never posts twice.
        let send = super::send(ctx, &token, chat_id, text, &ctx.settings["media"]);
        crate::rest::once(ctx, "telegram:send", async { send.await.map(Value::from) }).await?;
        Ok(NodeOutput::out(input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppTelegramSend)) }

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
        il_sdk::testkit::assert_manifest_valid(&super::AppTelegramSend);
    }

    #[tokio::test]
    async fn c02_telegram_sends_photo_from_file_store() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/botsecret/sendPhoto"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok":true,"result":{"message_id":7}})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppTelegramSend).credential("telegram", "cred");
        let file = harness.fakes.files.put("pic.png", "image/png", "pixels".into()).await.unwrap();
        let harness = harness.settings(json!({"chatId":"42","text":"caption","media":file.id,"_base":server.uri()}));
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "telegram".into(),
            secret: json!({"token":"secret"}),
            meta: json!({}),
        });
        assert!(harness.run().await.is_ok());
        assert!(harness.run().await.is_ok(), "a retry of the same run and item doesn't send again");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests.len(), 1);
        assert!(String::from_utf8_lossy(&requests[0].body).contains("pic.png"));
    }
}
