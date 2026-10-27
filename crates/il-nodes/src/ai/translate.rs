//! `ai.translate` — Translate.
//!

use super::common;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/translate.md");

use il_sdk::prelude::*;

/// Translate node.
#[derive(Debug, Default)]
pub struct AiTranslate;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.translate",
        Category::Ai,
        "Translate",
        "Translates text into another language, keeping formatting.",
    )
    .icon("translate")
    .status(NodeStatus::Ready)
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "input",
            "targetLanguage"
        ],
        "properties": {
            "input": {
                "type": "string",
                "title": "Input",
                "x-widget": "expression"
            },
            "targetLanguage": {
                "type": "string",
                "title": "Target language"
            },
            "preserveFormatting": {
                "type": "boolean",
                "title": "Keep formatting",
                "default": true
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
        "To Spanish",
        json!({
            "input": "{{ item.text }}",
            "targetLanguage": "Spanish"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiTranslate {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let system = format!(
            "{}\nTarget language: {}\nPreserve formatting: {}",
            PROMPT.trim(),
            common::text(&ctx.settings["targetLanguage"]),
            ctx.settings["preserveFormatting"].as_bool().unwrap_or(true)
        );
        let response =
            common::complete(ctx, ModelRole::Fast, system, common::input_text(&ctx.settings, &input), None).await?;
        Ok(common::output(&input, "translation", Value::String(response.content)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiTranslate)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiTranslate);
    }

    #[tokio::test]
    async fn b07_translate_uses_target_language() {
        let harness = NodeHarness::new(super::AiTranslate)
            .settings(json!({"input":"{{ item.text }}","targetLanguage":"Spanish"}))
            .input_json(vec![json!({"text":"Hello"})]);
        harness.fakes.models.push_text("Hola");
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["translation"], "Hola");
        assert!(harness.fakes.models.calls()[0].1.messages[0].content.contains("Spanish"));
    }
}
