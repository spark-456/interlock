//! `app.slack.send` — Send message (Slack).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C02 implements it.

use il_sdk::prelude::*;

/// Send message (Slack) node.
#[derive(Debug, Default)]
pub struct AppSlackSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.slack.send", Category::App, "Send message (Slack)", "Posts a message to a Slack channel.")
        .icon("message")
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSlackSend)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSlackSend);
    }
}
