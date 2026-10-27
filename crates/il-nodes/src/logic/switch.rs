//! `logic.switch` — Switch.
//!

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let value = &ctx.settings["value"];
        let cases =
            ctx.settings["cases"].as_array().ok_or_else(|| NodeError::Failed("cases must be an array".into()))?;
        let mut selected = "default";
        for case in cases {
            let name = case["name"].as_str().ok_or_else(|| NodeError::Failed("case name must be text".into()))?;
            if name.is_empty() || name == "default" {
                return Err(NodeError::Failed("case name must be nonempty and not default".into()));
            }
            if value == &case["equals"] {
                selected = name;
                break;
            }
        }
        Ok(NodeOutput::port(selected, input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicSwitch)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicSwitch);
    }

    #[tokio::test]
    async fn b06_switch_routes_first_match_and_default() {
        let output = NodeHarness::new(super::LogicSwitch)
            .settings(json!({"value":"{{ item.kind }}","cases":[{"name":"billing","equals":"billing"}]}))
            .input_json(vec![json!({"kind":"billing"}), json!({"kind":"other"})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["billing"].len(), 1);
        assert_eq!(output.ports["default"].len(), 1);
    }
}
