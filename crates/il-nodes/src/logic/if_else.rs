//! `logic.if` — If.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// If node.
#[derive(Debug, Default)]
pub struct LogicIf;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.if", Category::Logic, "If", "Sends each item to `true` or `false` based on a condition.")
        .icon("branch")
        .outputs(&["true", "false"])
        .settings_schema(json!({
            "type": "object",
            "required": [
                "condition"
            ],
            "properties": {
                "condition": {
                    "type": "string",
                    "title": "Condition",
                    "description": "An expression, e.g. `{{ item.confidence > 0.8 }}`.",
                    "x-widget": "expression"
                }
            }
        }))
        .example(
            "Confident answer",
            json!({
                "condition": "{{ item.confidence > 0.8 and item.label not in ['billing', 'complaint'] }}"
            }),
        )
});

#[async_trait]
impl NodeExecutor for LogicIf {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicIf)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicIf);
    }
}
