//! `ai.classify` — Classify.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiClassify)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiClassify);
    }
}
