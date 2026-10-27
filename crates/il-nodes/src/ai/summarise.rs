//! `ai.summarise` — Summarise.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

/// Summarise node.
#[derive(Debug, Default)]
pub struct AiSummarise;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.summarise", Category::Ai, "Summarise", "Summarises text into a paragraph or bullet points.")
        .icon("summary")
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiSummarise)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiSummarise);
    }
}
