//! `logic.wait` — Wait.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Wait node.
#[derive(Debug, Default)]
pub struct LogicWait;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.wait", Category::Logic, "Wait", "Pauses for a duration or until a time.")
        .icon("timer")
        .settings_schema(json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "title": "Mode",
                    "enum": [
                        "duration",
                        "until"
                    ],
                    "default": "duration",
                    "x-widget": "select"
                },
                "seconds": {
                    "type": "integer",
                    "title": "Seconds",
                    "default": 60,
                    "minimum": 1
                },
                "until": {
                    "type": "string",
                    "title": "Until",
                    "x-widget": "expression"
                }
            }
        }))
        .example(
            "One minute",
            json!({
                "mode": "duration",
                "seconds": 60
            }),
        )
});

#[async_trait]
impl NodeExecutor for LogicWait {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicWait)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicWait);
    }
}
