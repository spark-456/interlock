//! `ai.agent` — Agent.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

/// Agent node.
#[derive(Debug, Default)]
pub struct AiAgent;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.agent", Category::Ai, "Agent", "An assistant that answers using knowledge bases and tools in a loop, returning the answer, sources and activity.")
        .icon("bot")
        .cost(CostHint::Medium)
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
                    "default": "{{ item.message }}",
                    "x-widget": "expression"
                },
                "knowledgeBases": {
                    "type": "array",
                    "title": "Knowledge",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "tools": {
                    "type": "array",
                    "title": "Tools",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "maxSteps": {
                    "type": "integer",
                    "title": "Max steps",
                    "default": 8,
                    "minimum": 1,
                    "maximum": 20
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
        .example("Support agent", json!({
            "instructions": "You are Larkspur's support assistant. Answer from the help center; cite sources.",
            "knowledgeBases": [
                "kb_help_center"
            ],
            "tools": [
                "get_invoice"
            ]
        }))
});

#[async_trait]
impl NodeExecutor for AiAgent {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiAgent)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiAgent);
    }
}
