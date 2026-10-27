//! `ai.translate` — Translate.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiTranslate)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiTranslate);
    }
}
