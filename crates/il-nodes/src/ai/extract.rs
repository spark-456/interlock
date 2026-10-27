//! `ai.extract` — Extract fields.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

/// Extract fields node.
#[derive(Debug, Default)]
pub struct AiExtract;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("ai.extract", Category::Ai, "Extract fields", "Pulls structured fields out of text as JSON.")
        .icon("extract")
        .cost(CostHint::Low)
        .settings_schema(json!({
            "type": "object",
            "required": [
                "input",
                "fields"
            ],
            "properties": {
                "input": {
                    "type": "string",
                    "title": "Input",
                    "x-widget": "expression"
                },
                "fields": {
                    "type": "array",
                    "title": "Fields",
                    "items": {
                        "type": "object",
                        "title": "Field",
                        "properties": {
                            "name": {
                                "type": "string",
                                "title": "Name"
                            },
                            "type": {
                                "type": "string",
                                "title": "Type",
                                "enum": [
                                    "string",
                                    "number",
                                    "boolean",
                                    "date",
                                    "list"
                                ],
                                "default": "string",
                                "x-widget": "select"
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
            "Order details",
            json!({
                "input": "{{ item.body }}",
                "fields": [
                    {
                        "name": "orderNumber",
                        "type": "string"
                    },
                    {
                        "name": "refundRequested",
                        "type": "boolean"
                    }
                ]
            }),
        )
});

#[async_trait]
impl NodeExecutor for AiExtract {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiExtract)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiExtract);
    }
}
