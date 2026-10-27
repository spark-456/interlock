//! `app.discord.send` — Send message (Discord).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C02 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppDiscordSend)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppDiscordSend);
    }
}
