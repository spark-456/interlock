//! `ai.summarise` — Summarise.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/summarise.md");

use il_sdk::prelude::*;

/// Summarise node.
#[derive(Debug, Default)]
pub struct AiSummarise;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.summarise", Category::Ai, "Summarise", "Summarises text into a paragraph or bullet points.")
        .icon("summary")
        .status(NodeStatus::Ready)
        .cost(CostHint::Low)
        .settings_schema(json!({
            "type": "object",
            "required": [
                "input"
            ],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "Input",
                    "x-widget": "expression"
                },
                "format": {
                    "type": "string",
                    "title": "Format",
                    "enum": [
                        "paragraph",
                        "bullets"
                    ],
                    "default": "bullets",
                    "x-widget": "select"
                },
                "length": {
                    "type": "string",
                    "title": "Length",
                    "enum": [
                        "short",
                        "medium",
                        "long"
                    ],
                    "default": "short",
                    "x-widget": "select"
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
            "Summarise an email",
            json!({
                "input": "{{ item.body }}",
                "format": "bullets"
            }),
        )
});

#[async_trait]
impl NodeExecutor for AiSummarise {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let system = format!(
            "{}\nFormat: {}\nLength: {}",
            PROMPT.trim(),
            ctx.settings["format"].as_str().unwrap_or("bullets"),
            ctx.settings["length"].as_str().unwrap_or("short")
        );
        let response =
            common::complete(ctx, ModelRole::Fast, system, common::input_text(&ctx.settings, &input), None).await?;
        Ok(common::output(&input, "summary", Value::String(response.content)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiSummarise)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiSummarise);
    }

    #[tokio::test]
    async fn b07_summarise_returns_summary() {
        let harness = NodeHarness::new(super::AiSummarise)
            .settings(json!({"input":"{{ item.body }}","format":"bullets"}))
            .input_json(vec![json!({"body":"Long letter"})]);
        harness.fakes.models.push_text("- Short letter");
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["summary"], "- Short letter");
    }
}
