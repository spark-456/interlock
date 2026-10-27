//! `logic.if` — If.
//!

use il_sdk::prelude::*;

/// If node.
#[derive(Debug, Default)]
pub struct LogicIf;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.if", Category::Logic, "If", "Sends each item to `true` or `false` based on a condition.")
        .icon("branch")
        .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let condition = &ctx.settings["condition"];
        let yes = match condition {
            Value::Bool(value) => *value,
            Value::Null => false,
            Value::Number(number) => number.as_f64().is_some_and(|n| n != 0.0),
            Value::String(value) => !value.is_empty() && value != "false" && value != "0",
            Value::Array(values) => !values.is_empty(),
            Value::Object(values) => !values.is_empty(),
        };
        Ok(NodeOutput::port(if yes { "true" } else { "false" }, input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicIf)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicIf);
    }

    #[tokio::test]
    async fn b06_if_routes_each_item() {
        let output = NodeHarness::new(super::LogicIf)
            .settings(json!({"condition":"{{ item.score >= 80 }}"}))
            .input_json(vec![json!({"score":91}), json!({"score":20})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["true"][0].json["score"], 91);
        assert_eq!(output.ports["false"][0].json["score"], 20);
    }
}
