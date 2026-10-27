//! `knowledge.add` — Add to knowledge.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task K01 implements it.

use il_sdk::prelude::*;

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeAdd)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeAdd);
    }
}
