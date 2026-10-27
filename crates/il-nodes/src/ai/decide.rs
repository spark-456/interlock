//! `ai.decide` — Decide.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/decide.md");

use il_sdk::prelude::*;

/// Decide node.
#[derive(Debug, Default)]
pub struct AiDecide;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.decide",
        Category::Ai,
        "Decide",
        "Answers a question about each item and routes it: yes/no, or one of your labels (router).",
    )
    .icon("split")
    .status(NodeStatus::Ready)
    .outputs(&["yes", "no"])
    .dynamic_outputs("labels")
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "question"
        ],
        "properties": {
            "question": {
                "type": "string",
                "title": "Question",
                "x-widget": "longtext"
            },
            "input": {
                "type": "string",
                "title": "Input",
                "default": "{{ item }}",
                "x-widget": "expression"
            },
            "mode": {
                "type": "string",
                "title": "Mode",
                "enum": [
                    "yesno",
                    "labels"
                ],
                "default": "yesno",
                "x-widget": "select"
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
                },
                "default": []
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
        "Is it about billing?",
        json!({
            "question": "Is this message about an invoice, payment or refund?",
            "input": "{{ item.message }}",
            "mode": "yesno"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiDecide {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let labels: Vec<&str> = if ctx.settings["mode"].as_str().unwrap_or("yesno") == "labels" {
            ctx.settings["labels"].as_array().into_iter().flatten().filter_map(|entry| entry["name"].as_str()).collect()
        } else {
            vec!["yes", "no"]
        };
        if labels.is_empty() {
            return Err(NodeError::Failed("at least one decision label is required".into()));
        }
        let system = format!(
            "{}\nQuestion: {}\nLabels: {}",
            PROMPT.trim(),
            common::text(&ctx.settings["question"]),
            labels.join(", ")
        );
        let schema =
            json!({"type":"object","required":["label"],"properties":{"label":{"type":"string","enum":labels}}});
        let response =
            common::complete(ctx, ModelRole::Fast, system, common::input_text(&ctx.settings, &input), Some(schema))
                .await?;
        let parsed = response.json().map_err(|e| NodeError::Failed(e.to_string()))?;
        let label = parsed["label"].as_str().ok_or_else(|| NodeError::Failed("model returned no decision".into()))?;
        if !labels.contains(&label) {
            return Err(NodeError::Failed("model returned an unknown decision".into()));
        }
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["decision"] = json!(label);
        Ok(NodeOutput::port(label, vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiDecide)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiDecide);
    }

    #[tokio::test]
    async fn b07_decide_routes_yes_no() {
        let harness = NodeHarness::new(super::AiDecide)
            .settings(json!({"question":"Refund?","input":"{{ item.message }}"}))
            .input_json(vec![json!({"message":"Refund please"})]);
        harness.fakes.models.push_json(json!({"label":"yes"}));
        assert_eq!(harness.run().await.unwrap().ports["yes"][0].json["decision"], "yes");
    }
}
