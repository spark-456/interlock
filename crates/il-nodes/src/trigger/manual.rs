//! `trigger.manual` — Manual.
//!
//! Manual runs come from "Run"/"Test" (`FlowRunner::start` with this node).
//! Activation starts nothing in the background.

use il_sdk::prelude::*;

/// Manual node.
#[derive(Debug, Default)]
pub struct TriggerManual;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("trigger.manual", Category::Trigger, "Manual", "Starts the flow when you press Run or Test.")
        .icon("hand")
        .trigger()
        .status(NodeStatus::Ready)
        .settings_schema(json!({
            "type": "object",
            "properties": {}
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

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        // The engine emits a trigger's items as its output; executing it again passes them through.
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerManual {
    async fn start(&self, _ctx: il_sdk::TriggerCtx) -> Result<il_sdk::TriggerHandle, NodeError> {
        Ok(il_sdk::TriggerHandle::noop())
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
