//! `output.notify` — Send notification.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B08 implements it.

use il_sdk::prelude::*;

/// Send notification node.
#[derive(Debug, Default)]
pub struct OutputNotify;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "output.notify",
        Category::Output,
        "Send notification",
        "Sends a short notification in the app or to a channel set in settings.",
    )
    .icon("bell")
    .external()
    .settings_schema(json!({
        "type": "object",
        "required": [
            "message"
        ],
        "properties": {
            "channel": {
                "type": "string",
                "title": "Channel",
                "enum": [
                    "inapp",
                    "email",
                    "slack",
                    "discord",
                    "telegram"
                ],
                "default": "inapp",
                "x-widget": "select"
            },
            "title": {
                "type": "string",
                "title": "Title"
            },
            "message": {
                "type": "string",
                "title": "Message",
                "x-widget": "longtext"
            },
            "target": {
                "type": "string",
                "title": "Destination"
            }
        }
    }))
    .example(
        "Tell the team",
        json!({
            "channel": "inapp",
            "title": "Promos posted",
            "message": "{{ items | length }} posts went out."
        }),
    )
});

#[async_trait]
impl NodeExecutor for OutputNotify {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(OutputNotify)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::OutputNotify);
    }
}
