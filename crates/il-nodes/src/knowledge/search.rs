//! `knowledge.search` — Search knowledge.
//!
//! Searches a knowledge base and preserves source details.

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let kb = ctx.settings["knowledgeBase"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("knowledge base is required".into()))?;
        let query = ctx.settings["query"].as_str().ok_or_else(|| NodeError::Failed("query is required".into()))?;
        let top_k = ctx.settings["topK"].as_u64().unwrap_or(5).clamp(1, 20) as usize;
        let min_score = ctx.settings["minScore"].as_f64().unwrap_or(0.0) as f32;
        let hits: Vec<_> = ctx
            .knowledge()
            .search(kb, query, top_k)
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .into_iter()
            .filter(|hit| hit.score >= min_score)
            .collect();
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["hits"] = json!(hits);
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(KnowledgeSearch)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::knowledge::SearchHit;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::KnowledgeSearch);
    }

    #[tokio::test]
    async fn k01_search_filters_by_score() {
        let harness = NodeHarness::new(super::KnowledgeSearch)
            .settings(json!({"knowledgeBase":"kb_1","query":"refund","minScore":0.5}));
        harness.fakes.knowledge.hits.lock().unwrap().push(SearchHit {
            text: "Refund policy".into(),
            score: 0.8,
            source_id: "s1".into(),
            title: "Policy".into(),
            section: None,
            url: None,
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["hits"][0]["source_id"], "s1");
    }
}
