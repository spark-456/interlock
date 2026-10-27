//! `ai.write` — Write.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/write.md");

use il_sdk::prelude::*;

/// Write node.
#[derive(Debug, Default)]
pub struct AiWrite;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.write",
        Category::Ai,
        "Write",
        "Writes text from instructions and input: scripts, replies, posts, summaries in a given tone.",
    )
    .icon("pen")
    .status(NodeStatus::Ready)
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "instructions"
        ],
        "properties": {
            "instructions": {
                "type": "string",
                "title": "Instructions",
                "x-widget": "longtext"
            },
            "input": {
                "type": "string",
                "title": "Input",
                "default": "{{ item }}",
                "x-widget": "expression"
            },
            "tone": {
                "type": "string",
                "title": "Tone",
                "enum": [
                    "neutral",
                    "friendly",
                    "upbeat",
                    "formal"
                ],
                "default": "friendly",
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
                "default": "medium",
                "x-widget": "select"
            },
            "outputField": {
                "type": "string",
                "title": "Output field",
                "default": "text"
            },
            "model": {
                "type": "string",
                "title": "Model",
                "enum": [
                    "architect",
                    "agent",
                    "fast"
                ],
                "default": "agent",
                "x-widget": "model-role"
            }
        }
    }))
    .example(
        "Promo video script",
        json!({
            "instructions": "Write a 20-second upbeat promo script for this product.",
            "input": "{{ item.title }}: {{ item.description }}",
            "tone": "upbeat",
            "length": "short"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiWrite {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let system = format!(
            "{}\nInstructions: {}\nTone: {}\nLength: {}",
            PROMPT.trim(),
            common::text(&ctx.settings["instructions"]),
            ctx.settings["tone"].as_str().unwrap_or("friendly"),
            ctx.settings["length"].as_str().unwrap_or("medium")
        );
        let response =
            common::complete(ctx, ModelRole::Agent, system, common::input_text(&ctx.settings, &input), None).await?;
        let field = ctx.settings["outputField"].as_str().unwrap_or("text");
        Ok(common::output(&input, field, Value::String(response.content)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiWrite)) }

#[cfg(test)]
mod tests {
    use il_sdk::{services::ModelRole, testkit::NodeHarness};
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiWrite);
    }

    #[tokio::test]
    async fn b07_write_uses_role_and_preserves_input() {
        let harness = NodeHarness::new(super::AiWrite)
            .settings(json!({"instructions":"Write a greeting","input":"{{ item.name }}","outputField":"draft"}))
            .input_json(vec![json!({"name":"Ada"})]);
        harness.fakes.models.push_text("Hello, Ada!");
        let result = harness.run().await.unwrap();
        assert_eq!(result.ports["out"][0].json["draft"], "Hello, Ada!");
        assert_eq!(harness.fakes.models.calls()[0].0, ModelRole::Agent);
    }
}
