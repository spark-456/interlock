//! Platform tools exposed to an authenticated MCP client.

use il_core::dto::BuildRequest;
use il_core::{FlowKind, Item};
use il_sdk::registry::RouteCtx;
use il_sdk::services::StartRun;
use il_sdk::services::knowledge::SourceInput;
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::tool;
use crate::guide;

pub(super) fn tools() -> Vec<Tool> {
    vec![
        tool("list_flows", "List flows in this workspace", json!({"type":"object","properties":{}})),
        tool(
            "get_integration_guide",
            "Get a published flow's INTERLOCK.md guide",
            json!({"type":"object","properties":{"flow_id":{"type":"string"}},"required":["flow_id"]}),
        ),
        tool(
            "add_documents",
            "Add text documents to a knowledge base",
            json!({"type":"object","properties":{"kb_id":{"type":"string"},"documents":{"type":"array",
                "items":{"type":"object","properties":{"title":{"type":"string"},"text":{"type":"string"}},
                "required":["title","text"]}}},"required":["kb_id","documents"]}),
        ),
        tool(
            "create_flow_from_goal",
            "Start building a flow from a goal",
            json!({"type":"object","properties":{"goal":{"type":"string"}},"required":["goal"]}),
        ),
        tool(
            "run_flow",
            "Run a flow with JSON input",
            json!({"type":"object","properties":{"flow_id":{"type":"string"},"input":{}},"required":["flow_id"]}),
        ),
    ]
}

fn string_arg(arguments: &JsonObject, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{key} is required"))
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    name: &str,
    arguments: &JsonObject,
) -> Result<Value, String> {
    match name {
        "list_flows" => {
            let flows = ctx.services.store.list_flows(workspace_id).await.map_err(|e| e.to_string())?;
            Ok(json!(
                flows
                    .iter()
                    .map(|flow| json!({"id":flow.id,"name":flow.name,
                "kind":flow.kind,"version":flow.version}))
                    .collect::<Vec<_>>()
            ))
        }
        "get_integration_guide" => {
            let id = string_arg(arguments, "flow_id")?;
            let flow = ctx
                .services
                .store
                .get_flow(&id)
                .await
                .map_err(|e| e.to_string())?
                .filter(|flow| flow.workspace_id == workspace_id && flow.kind == FlowKind::App)
                .ok_or("flow not found")?;
            if !ctx.services.store.is_flow_active(&flow.id).await.map_err(|e| e.to_string())? {
                return Err("flow not found".into());
            }
            let base_url =
                ctx.config.pointer("/server/public_url").and_then(Value::as_str).unwrap_or("http://127.0.0.1:8080");
            Ok(Value::String(guide::generate(&flow, base_url)))
        }
        "add_documents" => {
            let kb_id = string_arg(arguments, "kb_id")?;
            if !ctx.services.knowledge.owns_base(workspace_id, &kb_id).await.map_err(|e| e.to_string())? {
                return Err("knowledge base not found".into());
            }
            let documents = arguments.get("documents").and_then(Value::as_array).ok_or("documents must be an array")?;
            let sources: Vec<_> = documents
                .iter()
                .map(|value| -> Result<SourceInput, &str> {
                    let title = value
                        .get("title")
                        .and_then(Value::as_str)
                        .filter(|text| !text.is_empty())
                        .ok_or("document title is required")?;
                    let text = value
                        .get("text")
                        .and_then(Value::as_str)
                        .filter(|text| !text.is_empty())
                        .ok_or("document text is required")?;
                    Ok(SourceInput::Text { title: title.into(), text: text.into() })
                })
                .collect::<Result<_, _>>()?;
            let ids = ctx.services.knowledge.add_sources(&kb_id, sources).await.map_err(|e| e.to_string())?;
            Ok(json!({"source_ids":ids}))
        }
        "create_flow_from_goal" => {
            let goal = string_arg(arguments, "goal")?;
            let id = ctx
                .architect
                .start_build(workspace_id, BuildRequest { goal, kind: None, attachments: vec![], hints: vec![] })
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({"build_id":id}))
        }
        "run_flow" => {
            let id = string_arg(arguments, "flow_id")?;
            let flow = ctx
                .services
                .store
                .get_flow(&id)
                .await
                .map_err(|e| e.to_string())?
                .filter(|flow| flow.workspace_id == workspace_id)
                .ok_or("flow not found")?;
            let input = arguments.get("input").cloned().unwrap_or(Value::Null);
            let outcome = ctx
                .runner
                .invoke(StartRun {
                    flow_id: flow.id,
                    version: None,
                    trigger: "mcp".into(),
                    items: vec![Item::json(input)],
                    trigger_node: None,
                })
                .await
                .map_err(|e| e.to_string())?;
            Ok(json!({"run_id":outcome.run.id,"response":outcome.response}))
        }
        _ => Err("tool not found".into()),
    }
}
