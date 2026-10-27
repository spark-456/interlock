//! `knowledge.search` — Search knowledge.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task K01 implements it.

use il_sdk::prelude::*;

/// Search knowledge node.
#[derive(Debug, Default)]
pub struct KnowledgeSearch;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "knowledge.search",
        Category::Knowledge,
        "Search knowledge",
        "Finds the most relevant passages for a query, with sources.",
    )
    .icon("search")
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "knowledgeBase",
            "query"
        ],
        "properties": {
            "knowledgeBase": {
                "type": "string",
                "title": "Knowledge base"
            },
            "query": {
                "type": "string",
                "title": "Query",
                "x-widget": "expression"
            },
            "topK": {
                "type": "integer",
                "title": "Results",
                "default": 5,
                "minimum": 1,
                "maximum": 20
            },
            "minScore": {
                "type": "number",
                "title": "Minimum score",
                "default": 0.0,
                "minimum": 0.0,
                "maximum": 1.0
            }
        }
    }))
    .example(
        "Find policy",
        json!({
            "knowledgeBase": "kb_help_center",
            "query": "{{ item.body }}",
            "topK": 5
        }),
    )
});

#[async_trait]
impl NodeExecutor for KnowledgeSearch {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeSearch)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeSearch);
    }
}
