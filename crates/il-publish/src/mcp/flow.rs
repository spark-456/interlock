//! Tools exposed by one published app flow.

use std::collections::BTreeSet;
use std::sync::Arc;

use il_core::{Flow, Item};
use il_sdk::registry::RouteCtx;
use il_sdk::services::{AgentTool, StartRun};
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::tool;
use crate::openai::slug;

fn bases(flow: &Flow) -> BTreeSet<&str> {
    flow.nodes
        .iter()
        .filter(|node| !node.disabled)
        .flat_map(|node| {
            let direct = (node.node_type == "knowledge.search")
                .then(|| node.settings.get("knowledgeBase").and_then(Value::as_str))
                .flatten();
            let agent = (node.node_type == "ai.agent")
                .then(|| node.settings.get("knowledgeBases").and_then(Value::as_array))
                .flatten();
            direct.into_iter().chain(agent.into_iter().flatten().filter_map(Value::as_str))
        })
        .collect()
}

fn selected_tools(flow: &Flow) -> BTreeSet<&str> {
    flow.nodes
        .iter()
        .filter(|node| !node.disabled)
        .flat_map(|node| {
            let operation = (node.node_type == "data.openapi")
                .then(|| node.settings.get("operationId").and_then(Value::as_str))
                .flatten();
            let agent =
                (node.node_type == "ai.agent").then(|| node.settings.get("tools").and_then(Value::as_array)).flatten();
            operation.into_iter().chain(agent.into_iter().flatten().filter_map(Value::as_str))
        })
        .collect()
}

pub(super) fn tools(flow: &Flow, available: &[Arc<dyn AgentTool>]) -> Vec<Tool> {
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
    let selected = selected_tools(flow);
    let mut seen: BTreeSet<String> = tools.iter().map(|tool| tool.name.to_string()).collect();
    for agent_tool in available {
        if selected.contains(agent_tool.name()) && seen.insert(agent_tool.name().to_owned()) {
            tools.push(tool(agent_tool.name(), agent_tool.description(), agent_tool.schema()));
        }
    }
    tools
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    flow: &Flow,
    name: &str,
    arguments: &JsonObject,
    available: &[Arc<dyn AgentTool>],
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
                only: None,
            })
            .await
            .map_err(|e| e.to_string())?;
        return Ok(json!({"run_id":outcome.run.id,"response":outcome.response}));
    }
    let selected = selected_tools(flow);
    let knowledge_name = name.strip_prefix("search_").is_some_and(|id| bases(flow).contains(id));
    if let Some(agent_tool) =
        available.iter().find(|tool| tool.name() == name && selected.contains(name) && !knowledge_name)
    {
        return agent_tool.invoke(Value::Object(arguments.clone())).await.map_err(|e| e.to_string());
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
