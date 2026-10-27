//! `trigger.manual` — Manual.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

use il_sdk::prelude::*;

/// Manual node.
#[derive(Debug, Default)]
pub struct TriggerManual;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.manual",
        Category::Trigger,
        "Manual",
        "Starts the flow when you press Run or Test. Optional sample items are used for test runs.",
    )
    .icon("hand")
    .trigger()
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "sampleItems": {
                "type": "array",
                "title": "Sample items",
                "items": {
                    "type": "object"
                },
                "default": []
            }
        }
    }))
    .example(
        "Run by hand",
        json!({
            "sampleItems": [
                {
                    "title": "Larkspur Tote"
                }
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerManual {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerManual)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerManual);
    }
}
