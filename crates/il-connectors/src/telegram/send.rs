//! `app.telegram.send` — Send message (Telegram).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C02 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppTelegramSend)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppTelegramSend);
    }
}
