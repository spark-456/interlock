//! `logic.stop` — Stop.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Stop node.
#[derive(Debug, Default)]
pub struct LogicStop;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.stop", Category::Logic, "Stop", "Ends this branch, as success or with an error message.")
        .icon("stop")
        .outputs(&[])
        .settings_schema(json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "title": "Status",
                    "enum": [
                        "success",
                        "error"
                    ],
                    "default": "success",
                    "x-widget": "select"
                },
                "reason": {
                    "type": "string",
                    "title": "Reason"
                }
            }
        }))
        .example(
            "Ignore spam",
            json!({
                "status": "success",
                "reason": "Spam"
            }),
        )
});

#[async_trait]
impl NodeExecutor for LogicStop {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicStop)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicStop);
    }
}
