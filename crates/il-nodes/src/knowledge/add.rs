//! `knowledge.add` — Add to knowledge.
//!
//! Adds text to a knowledge base.

use il_sdk::prelude::*;
use il_sdk::services::knowledge::SourceInput;

/// Add to knowledge node.
#[derive(Debug, Default)]
pub struct KnowledgeAdd;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "knowledge.add",
        Category::Knowledge,
        "Add to knowledge",
        "Adds text or a file from each item to a knowledge base.",
    )
    .icon("book-plus")
    .status(NodeStatus::Ready)
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "knowledgeBase",
            "content"
        ],
        "properties": {
            "knowledgeBase": {
                "type": "string",
                "title": "Knowledge base"
            },
            "content": {
                "type": "string",
                "title": "Content",
                "x-widget": "expression"
            },
            "title": {
                "type": "string",
                "title": "Title",
                "x-widget": "expression"
            },
            "markAs": {
                "type": "string",
                "title": "Use as",
                "enum": [
                    "knowledge",
                    "instructions"
                ],
                "default": "knowledge",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Save answer",
        json!({
            "knowledgeBase": "kb_help_center",
            "content": "{{ item.answer }}",
            "title": "{{ item.subject }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for KnowledgeAdd {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let kb = ctx.settings["knowledgeBase"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("knowledge base is required".into()))?;
        let content =
            ctx.settings["content"].as_str().ok_or_else(|| NodeError::Failed("content is required".into()))?;
        let title = ctx.settings["title"].as_str().unwrap_or("Added text");
        let source = if ctx.settings["markAs"] == "instructions" {
            SourceInput::InstructionsText { title: title.into(), text: content.into() }
        } else {
            SourceInput::Text { title: title.into(), text: content.into() }
        };
        let ids = ctx.knowledge().add_sources(kb, vec![source]).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["sourceIds"] = json!(ids);
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeAdd)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeAdd);
    }

    #[tokio::test]
    async fn k01_add_returns_source_id() {
        let harness = NodeHarness::new(super::KnowledgeAdd)
            .settings(json!({"knowledgeBase":"kb_1","content":"{{ item.body }}"}))
            .input_json(vec![json!({"body":"Policy"})]);
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["sourceIds"].as_array().unwrap().len(), 1);
    }
}
