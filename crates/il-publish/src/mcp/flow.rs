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
    for node in flow.nodes.iter().filter(|node| node.node_type == "trigger.mcp_call" && !node.disabled) {
        let Some(name) = node.settings.get("toolName").and_then(Value::as_str).filter(|name| !name.is_empty()) else {
            continue;
        };
        if seen.insert(name.to_owned()) {
            let description = node.settings.get("toolDescription").and_then(Value::as_str).unwrap_or("Run this flow");
            let schema = node.settings.get("inputSchema").filter(|value| value.is_object()).cloned().unwrap_or_else(
                || json!({"type":"object","properties":{"message":{"type":"string"}},"required":["message"]}),
            );
            tools.push(tool(name, description, schema));
        }
    }
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
        // What an assistant calling this tool needs: the reply and where it came from.
        let reply = outcome.response.first().map(|item| &item.json);
        let answer = reply
            .and_then(|j| j.get("answer").or_else(|| j.get("body")).or_else(|| j.get("message")).or(Some(j)))
            .cloned()
            .unwrap_or(Value::Null);
        let sources = reply.and_then(|j| j.get("sources")).cloned().unwrap_or_else(|| json!([]));
        return Ok(json!({"answer":answer,"sources":sources,"run_id":outcome.run.id}));
    }
    if let Some(node) = flow.nodes.iter().find(|node| {
        node.node_type == "trigger.mcp_call"
            && !node.disabled
            && node.settings.get("toolName").and_then(Value::as_str) == Some(name)
    }) {
        let outcome = ctx
            .runner
            .invoke(StartRun {
                flow_id: flow.id.clone(),
                version: None,
                trigger: "mcp".into(),
                items: vec![Item::json(Value::Object(arguments.clone()))],
                trigger_node: Some(node.id.clone()),
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
    let Some(named) = name.strip_prefix("search_").filter(|id| bases(flow).contains(*id)) else {
        return Err("tool not found".into());
    };
    // Agents name a knowledge step of the flow; its base is the one the step last built.
    let kb_id = match flow.node(named).filter(|node| node.node_type == "knowledge.base") {
        Some(step) => {
            let key = format!("kb:node:{workspace_id}:{}:{}", flow.id, step.id);
            let saved = ctx.services.store.kv_get(&key).await.map_err(|e| e.to_string())?;
            saved
                .and_then(|saved| saved["knowledgeBase"].as_str().map(str::to_owned))
                .ok_or("this knowledge base hasn't been built yet; run the app once")?
        }
        None => named.to_owned(),
    };
    let kb_id = kb_id.as_str();
    if !ctx.services.knowledge.owns_base(workspace_id, kb_id).await.map_err(|e| e.to_string())? {
        return Err("knowledge base not found".into());
    }
    let query =
        arguments.get("query").and_then(Value::as_str).filter(|text| !text.is_empty()).ok_or("query is required")?;
    let top_k = arguments.get("top_k").and_then(Value::as_u64).unwrap_or(5).clamp(1, 20) as usize;
    let hits = ctx.services.knowledge.search(kb_id, query, top_k).await.map_err(|e| e.to_string())?;
    Ok(json!({"hits":hits}))
}

#[cfg(test)]
mod tests {
    use super::*;
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};

    #[tokio::test]
    async fn n02_mcp_trigger_uses_configured_name_description_and_schema() {
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws","version":1,
            "name":"Invoices","kind":"app","nodes":[{"id":"n_mcp","type":"trigger.mcp_call","name":"MCP",
            "settings":{"toolName":"lookup_invoice","toolDescription":"Look up an invoice",
                "inputSchema":{"type":"object","properties":{"invoiceId":{"type":"string"}},"required":["invoiceId"]}}}],"edges":[]})).unwrap();
        let listed = tools(&flow, &[]);
        let configured = listed.iter().find(|tool| tool.name == "lookup_invoice").unwrap();
        let details = serde_json::to_value(configured).unwrap();
        assert_eq!(details["description"], "Look up an invoice");
        assert_eq!(details["inputSchema"]["required"], json!(["invoiceId"]));

        let fakes = FakeServices::default();
        let runner = Arc::new(FakeRunner::default());
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: runner.clone(),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            activation: std::sync::Arc::new(il_sdk::services::NoActivation),
            config: Value::Null,
        };
        let args = serde_json::from_value(json!({"invoiceId":"inv_1"})).unwrap();
        let response = call(&ctx, "ws", &flow, "lookup_invoice", &args, &[]).await.unwrap();
        assert_eq!(response["response"][0]["json"]["invoiceId"], "inv_1");
        assert_eq!(runner.started()[0].trigger_node.as_deref(), Some("n_mcp"));
    }
}
