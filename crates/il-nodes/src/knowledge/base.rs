//! `knowledge.base` — Knowledge base.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task K01 implements it.

use il_sdk::prelude::*;

/// Knowledge base node.
#[derive(Debug, Default)]
pub struct KnowledgeBase;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "knowledge.base",
        Category::Knowledge,
        "Knowledge base",
        "A searchable collection of documents, websites and transcripts. Connect it to an Agent or Search knowledge.",
    )
    .icon("book")
    .inputs(&[])
    .outputs(&["kb"])
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "name"
        ],
        "properties": {
            "name": {
                "type": "string",
                "title": "Name"
            },
            "sources": {
                "type": "array",
                "title": "Sources",
                "items": {
                    "type": "object",
                    "title": "Source",
                    "properties": {
                        "kind": {
                            "type": "string",
                            "title": "Kind",
                            "enum": [
                                "file",
                                "url",
                                "text"
                            ],
                            "x-widget": "select"
                        },
                        "value": {
                            "type": "string",
                            "title": "Value"
                        },
                        "role": {
                            "type": "string",
                            "title": "Use as",
                            "enum": [
                                "knowledge",
                                "instructions"
                            ],
                            "default": "knowledge",
                            "x-widget": "select"
                        }
                    },
                    "required": [
                        "kind",
                        "value"
                    ]
                },
                "default": []
            },
            "crawlLimit": {
                "type": "integer",
                "title": "Pages per website",
                "default": 50,
                "minimum": 1,
                "maximum": 50
            }
        }
    }))
    .example(
        "Help center",
        json!({
            "name": "Larkspur help center",
            "sources": [
                {
                    "kind": "url",
                    "value": "https://help.larkspur.example"
                }
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for KnowledgeBase {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeBase)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeBase);
    }
}
