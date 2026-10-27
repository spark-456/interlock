//! Tools exposed by one published app flow.

use std::collections::BTreeSet;

use il_core::{Flow, Item};
use il_sdk::registry::RouteCtx;
use il_sdk::services::StartRun;
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::tool;
use crate::openai::slug;

fn bases(flow: &Flow) -> BTreeSet<&str> {
    flow.nodes
        .iter()
        .filter(|node| node.node_type == "knowledge.search" && !node.disabled)
        .filter_map(|node| node.settings.get("knowledgeBase").and_then(Value::as_str))
        .collect()
}

pub(super) fn tools(flow: &Flow) -> Vec<Tool> {
    let mut tools = vec![tool(
        &format!("ask_{}", slug(flow)),
        "Ask this published app",
        json!({"type":"object","properties":{"message":{"type":"string"}},"required":["message"]}),
    )];
    for kb_id in bases(flow) {
        tools.push(tool(&format!("search_{kb_id}"), "Search a knowledge base used by this app",
            json!({"type":"object","properties":{"query":{"type":"string"},"top_k":{"type":"integer","minimum":1,"maximum":20}},
                "required":["query"]})));
    }
    tools
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    flow: &Flow,
    name: &str,
    arguments: &JsonObject,
) -> Result<Value, String> {
    if name == format!("ask_{}", slug(flow)) {
        let message = arguments
            .get("message")
            .and_then(Value::as_str)
            .filter(|text| !text.is_empty())
            .ok_or("message is required")?;
        let outcome = ctx
            .runner
            .invoke(StartRun {
                flow_id: flow.id.clone(),
                version: None,
                trigger: "mcp".into(),
                items: vec![Item::json(json!({"message":message}))],
                trigger_node: None,
            })
            .await
            .map_err(|e| e.to_string())?;
        return Ok(json!({"run_id":outcome.run.id,"response":outcome.response}));
    }
    let Some(kb_id) = name.strip_prefix("search_").filter(|id| bases(flow).contains(*id)) else {
        return Err("tool not found".into());
    };
    if !ctx.services.knowledge.owns_base(workspace_id, kb_id).await.map_err(|e| e.to_string())? {
        return Err("knowledge base not found".into());
    }
    let query =
        arguments.get("query").and_then(Value::as_str).filter(|text| !text.is_empty()).ok_or("query is required")?;
    let top_k = arguments.get("top_k").and_then(Value::as_u64).unwrap_or(5).clamp(1, 20) as usize;
    let hits = ctx.services.knowledge.search(kb_id, query, top_k).await.map_err(|e| e.to_string())?;
    Ok(json!({"hits":hits}))
}
