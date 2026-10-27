//! `ai.classify` — Classify.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/classify.md");

use il_sdk::prelude::*;

/// Classify node.
#[derive(Debug, Default)]
pub struct AiClassify;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.classify",
        Category::Ai,
        "Classify",
        "Sorts each item into one of your labels; each label becomes an output.",
    )
    .icon("tag")
    .status(NodeStatus::Ready)
    .outputs(&[])
    .dynamic_outputs("labels")
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "input",
            "labels"
        ],
        "properties": {
            "input": {
                "type": "string",
                "title": "Input",
                "x-widget": "expression"
            },
            "labels": {
                "type": "array",
                "title": "Labels",
                "items": {
                    "type": "object",
                    "title": "Label",
                    "properties": {
                        "name": {
                            "type": "string",
                            "title": "Name"
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
        "Support email type",
        json!({
            "input": "{{ item.subject }}\n{{ item.body }}",
            "labels": [
                {
                    "name": "question"
                },
                {
                    "name": "billing"
                },
                {
                    "name": "complaint"
                },
                {
                    "name": "spam"
                }
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiClassify {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let labels =
            ctx.settings["labels"].as_array().ok_or_else(|| NodeError::Failed("labels must be a list".into()))?;
        let names: Vec<&str> = labels.iter().filter_map(|entry| entry["name"].as_str()).collect();
        if names.is_empty() {
            return Err(NodeError::Failed("at least one label is required".into()));
        }
        let schema =
            json!({"type":"object","required":["label"],"properties":{"label":{"type":"string","enum":names}}});
        let system = format!(
            "{}\nLabels: {}",
            PROMPT.trim(),
            serde_json::to_string(labels).map_err(|e| NodeError::Failed(e.to_string()))?
        );
        let response =
            common::complete(ctx, ModelRole::Fast, system, common::input_text(&ctx.settings, &input), Some(schema))
                .await?;
        let parsed = response.json().map_err(|e| NodeError::Failed(e.to_string()))?;
        let label = parsed["label"].as_str().ok_or_else(|| NodeError::Failed("model returned no label".into()))?;
        if !names.contains(&label) {
            return Err(NodeError::Failed("model returned an unknown label".into()));
        }
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["label"] = json!(label);
        Ok(NodeOutput::port(label, vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiClassify)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiClassify);
    }

    #[tokio::test]
    async fn b07_classify_routes_to_named_port() {
        let harness = NodeHarness::new(super::AiClassify)
            .settings(json!({"input":"{{ item.subject }}","labels":[{"name":"billing"},{"name":"support"}]}))
            .input_json(vec![json!({"subject":"Invoice"})]);
        harness.fakes.models.push_json(json!({"label":"billing"}));
        assert_eq!(harness.run().await.unwrap().ports["billing"][0].json["label"], "billing");
    }
}
