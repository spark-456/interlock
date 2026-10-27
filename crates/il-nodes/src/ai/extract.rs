//! `ai.extract` — Extract fields.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/extract.md");

use il_sdk::prelude::*;

/// Extract fields node.
#[derive(Debug, Default)]
pub struct AiExtract;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.extract", Category::Ai, "Extract fields", "Pulls structured fields out of text as JSON.")
        .icon("extract")
        .status(NodeStatus::Ready)
        .cost(CostHint::Low)
        .settings_schema(json!({
            "type": "object",
            "required": [
                "input",
                "fields"
            ],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "Input",
                    "x-widget": "expression"
                },
                "fields": {
                    "type": "array",
                    "title": "Fields",
                    "items": {
                        "type": "object",
                        "title": "Field",
                        "properties": {
                            "name": {
                                "type": "string",
                                "title": "Name"
                            },
                            "type": {
                                "type": "string",
                                "title": "Type",
                                "enum": [
                                    "string",
                                    "number",
                                    "boolean",
                                    "date",
                                    "list"
                                ],
                                "default": "string",
                                "x-widget": "select"
                            },
                            "description": {
                                "type": "string",
                                "title": "Description"
                            }
                        },
                        "required": [
                            "name"
                        ]
                    }
                },
                "model": {
                    "type": "string",
                    "title": "Model",
                    "enum": [
                        "architect",
                        "agent",
                        "fast"
                    ],
                    "default": "fast",
                    "x-widget": "model-role"
                }
            }
        }))
        .example(
            "Order details",
            json!({
                "input": "{{ item.body }}",
                "fields": [
                    {
                        "name": "orderNumber",
                        "type": "string"
                    },
                    {
                        "name": "refundRequested",
                        "type": "boolean"
                    }
                ]
            }),
        )
});

#[async_trait]
impl NodeExecutor for AiExtract {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let fields =
            ctx.settings["fields"].as_array().ok_or_else(|| NodeError::Failed("fields must be a list".into()))?;
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();
        for field in fields {
            let name = field["name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| NodeError::Failed("field name is required".into()))?;
            let kind = match field["type"].as_str().unwrap_or("string") {
                "string" | "date" => json!({"type":["string","null"]}),
                "number" => json!({"type":["number","null"]}),
                "boolean" => json!({"type":["boolean","null"]}),
                "list" => json!({"type":["array","null"],"items":{"type":"string"}}),
                _ => return Err(NodeError::Failed("unknown field type".into())),
            };
            required.push(name.to_string());
            properties.insert(name.into(), kind);
        }
        let schema = json!({"type":"object","properties":properties,"required":required,"additionalProperties":false});
        let system = format!(
            "{}\nFields: {}",
            PROMPT.trim(),
            serde_json::to_string(fields).map_err(|e| NodeError::Failed(e.to_string()))?
        );
        let response =
            common::complete(ctx, ModelRole::Fast, system, common::input_text(&ctx.settings, &input), Some(schema))
                .await?;
        let value = response.json().map_err(|e| NodeError::Failed(e.to_string()))?;
        if !value.is_object() {
            return Err(NodeError::Failed("model returned non-object fields".into()));
        }
        Ok(common::output(&input, "fields", value))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiExtract)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiExtract);
    }

    #[tokio::test]
    async fn b07_extract_requests_schema_and_returns_fields() {
        let harness = NodeHarness::new(super::AiExtract)
            .settings(json!({"input":"{{ item.body }}","fields":[{"name":"orderNumber","type":"string"}]}))
            .input_json(vec![json!({"body":"Order A12"})]);
        harness.fakes.models.push_json(json!({"orderNumber":"A12"}));
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["fields"]["orderNumber"], "A12");
        assert!(harness.fakes.models.calls()[0].1.json_schema.is_some());
    }
}
