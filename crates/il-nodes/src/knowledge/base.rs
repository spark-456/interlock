//! `knowledge.base` — Knowledge base.
//!
//! Creates and populates a knowledge base.

use il_sdk::prelude::*;
use il_sdk::services::knowledge::SourceInput;

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        let name = ctx.settings["name"].as_str().ok_or_else(|| NodeError::Failed("name is required".into()))?;
        let id = ctx
            .knowledge()
            .create_base(&ctx.run.workspace_id, name)
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let limit = ctx.settings["crawlLimit"].as_u64().unwrap_or(50).clamp(1, 50) as u32;
        let mut sources = Vec::new();
        for source in ctx.settings["sources"].as_array().into_iter().flatten() {
            let value = source["value"].as_str().ok_or_else(|| NodeError::Failed("source value is required".into()))?;
            sources.push(match source["kind"].as_str() {
                Some("url") => SourceInput::Url { url: value.into(), crawl_limit: limit },
                Some("file") if source["role"] == "instructions" => {
                    SourceInput::MarkedFile { file_id: value.into(), name: value.into(), role: "instructions".into() }
                }
                Some("file") => SourceInput::File { file_id: value.into(), name: value.into() },
                Some("text") if source["role"] == "instructions" => {
                    SourceInput::InstructionsText { title: name.into(), text: value.into() }
                }
                Some("text") => SourceInput::Text { title: name.into(), text: value.into() },
                _ => return Err(NodeError::Failed("unknown source kind".into())),
            });
        }
        let source_ids = if sources.is_empty() {
            Vec::new()
        } else {
            ctx.knowledge().add_sources(&id, sources).await.map_err(|e| NodeError::Failed(e.to_string()))?
        };
        Ok(NodeOutput::port("kb", vec![Item::json(json!({"knowledgeBase":id,"sources":source_ids}))]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeBase)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeBase);
    }

    #[tokio::test]
    async fn k01_base_creates_and_populates() {
        let harness = NodeHarness::new(super::KnowledgeBase)
            .settings(json!({"name":"Help","sources":[{"kind":"text","value":"Refunds in 30 days"}]}));
        let out = harness.run().await.unwrap();
        assert!(out.ports["kb"][0].json["knowledgeBase"].as_str().unwrap().starts_with("kb_"));
        assert_eq!(out.ports["kb"][0].json["sources"].as_array().unwrap().len(), 1);
    }
}
