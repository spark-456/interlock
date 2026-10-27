//! `logic.switch` — Switch.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Switch node.
#[derive(Debug, Default)]
pub struct LogicSwitch;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "logic.switch",
        Category::Logic,
        "Switch",
        "Routes each item to the first case whose value matches; others go to `default`.",
    )
    .icon("switch")
    .outputs(&["default"])
    .dynamic_outputs("cases")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "value",
            "cases"
        ],
        "properties": {
            "value": {
                "type": "string",
                "title": "Value",
                "x-widget": "expression"
            },
            "cases": {
                "type": "array",
                "title": "Cases",
                "items": {
                    "type": "object",
                    "title": "Case",
                    "properties": {
                        "name": {
                            "type": "string",
                            "title": "Output name"
                        },
                        "equals": {
                            "type": "string",
                            "title": "Equals"
                        }
                    },
                    "required": [
                        "name",
                        "equals"
                    ]
                }
            }
        }
    }))
    .example(
        "By label",
        json!({
            "value": "{{ item.label }}",
            "cases": [
                {
                    "name": "question",
                    "equals": "question"
                },
                {
                    "name": "spam",
                    "equals": "spam"
                }
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for LogicSwitch {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicSwitch)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicSwitch);
    }
}
