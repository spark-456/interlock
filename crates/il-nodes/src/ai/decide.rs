//! `ai.decide` — Decide.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiDecide)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiDecide);
    }
}
