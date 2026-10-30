//! Reading, editing, refining and switching flows on.

use il_core::{Flow, FlowKind};
use il_sdk::registry::RouteCtx;
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::{owned_flow, string_arg, strings_arg};
use crate::mcp::tool;
use crate::openai::slug;
use crate::{export, guide};

pub(super) fn tools() -> Vec<Tool> {
    let id = json!({"type": "string", "description": "Flow id from list_flows"});
    vec![
        tool(
            "list_flows",
            "List the workspace's flows: id, name, kind (automation or app), whether it is on, and its goal.",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "get_flow",
            "Get a flow as JSON: its steps (type, name, settings), the edges between them and groups.",
            json!({"type": "object", "properties": {"flow_id": id}, "required": ["flow_id"]}),
        ),
        tool(
            "update_flow",
            "Save an edited flow. Pass the whole flow JSON from get_flow with your changes; saving makes a new version.",
            json!({"type": "object", "properties": {"flow_id": id, "flow": {"type": "object"}},
                "required": ["flow_id", "flow"]}),
        ),
        tool(
            "refine_flow",
            "Change a flow by describing the change in plain words (Interlock's AI edits it). Name step ids in \
             `focus` to change only those steps. Returns the proposal; with `apply: true` it is saved.",
            json!({"type": "object", "properties": {
                "flow_id": id,
                "request": {"type": "string", "description": "e.g. \"ask me in Slack before posting\""},
                "focus": {"type": "array", "items": {"type": "string"}, "description": "Step ids the change is about"},
                "apply": {"type": "boolean", "description": "Save the change (default false: just propose)"}
            }, "required": ["flow_id", "request"]}),
        ),
        tool(
            "set_flow_active",
            "Turn a flow on (its triggers start; an app becomes reachable by chat, API and MCP) or off.",
            json!({"type": "object", "properties": {"flow_id": id, "active": {"type": "boolean"}},
                "required": ["flow_id", "active"]}),
        ),
        tool(
            "get_integration_guide",
            "Get a published app's INTERLOCK.md: how to call it over the OpenAI-compatible API, widget and MCP.",
            json!({"type": "object", "properties": {"flow_id": id}, "required": ["flow_id"]}),
        ),
        tool(
            "export_flow",
            "Export a flow as code to run self-hosted: returns a command that downloads a zip (flow JSON, prompts,              files, compose.yaml, .env.example, README.md, INTERLOCK.md) into the current folder, the files it              holds, and the settings it needs. Nothing secret is in it.",
            json!({"type": "object", "properties": {"flow_id": id}, "required": ["flow_id"]}),
        ),
    ]
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    name: &str,
    arguments: &JsonObject,
) -> Option<Result<Value, String>> {
    Some(match name {
        "list_flows" => list(ctx, workspace_id).await,
        "get_flow" => match string_arg(arguments, "flow_id") {
            Ok(id) => owned_flow(ctx, workspace_id, &id).await.map(|flow| json!(flow)),
            Err(e) => Err(e),
        },
        "update_flow" => update(ctx, workspace_id, arguments).await,
        "refine_flow" => refine(ctx, workspace_id, arguments).await,
        "set_flow_active" => set_active(ctx, workspace_id, arguments).await,
        "get_integration_guide" => integration_guide(ctx, workspace_id, arguments).await,
        "export_flow" => export_flow(ctx, workspace_id, arguments).await,
        _ => return None,
    })
}

async fn list(ctx: &RouteCtx, workspace_id: &str) -> Result<Value, String> {
    let store = &ctx.services.store;
    let mut out = Vec::new();
    for flow in store.list_flows(workspace_id).await.map_err(|e| e.to_string())? {
        let active = store.is_flow_active(&flow.id).await.map_err(|e| e.to_string())?;
        out.push(json!({"id": flow.id, "name": flow.name, "kind": flow.kind, "version": flow.version,
            "active": active, "goal": flow.meta.goal, "steps": flow.nodes.len()}));
    }
    Ok(json!(out))
}

async fn update(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let saved = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let edited = arguments.get("flow").cloned().ok_or("flow is required")?;
    let flow: Flow = serde_json::from_value(edited).map_err(|e| format!("flow is not valid: {e}"))?;
    save(ctx, Flow { id: saved.id, workspace_id: saved.workspace_id, ..flow }).await
}

async fn save(ctx: &RouteCtx, mut flow: Flow) -> Result<Value, String> {
    if let Some(edge) = flow.edges.iter().find(|e| flow.node(&e.from.node).is_none() || flow.node(&e.to.node).is_none())
    {
        return Err(format!("edge {} connects a step that isn't in the flow", edge.id));
    }
    flow.version = ctx.services.store.put_flow(&flow).await.map_err(|e| e.to_string())?;
    Ok(json!({"flow_id": flow.id, "version": flow.version, "saved": true}))
}

async fn refine(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let request = string_arg(arguments, "request")?;
    let diff = ctx.architect.ask(&flow, &request, &strings_arg(arguments, "focus")).await.map_err(|e| e.to_string())?;
    let mut proposed = json!(flow);
    for op in diff.json_patch.as_array().into_iter().flatten() {
        if let (Some(path), Some(value)) = (op["path"].as_str(), op.get("value")) {
            proposed[path.trim_start_matches('/')] = value.clone();
        }
    }
    let proposed: Flow = serde_json::from_value(proposed).map_err(|e| e.to_string())?;
    let changed: Vec<Value> = proposed
        .nodes
        .iter()
        .filter(|n| flow.node(&n.id).is_none_or(|old| old != *n))
        .map(|n| json!({"id": n.id, "type": n.node_type, "name": n.name, "settings": n.settings}))
        .collect();
    let removed: Vec<&str> =
        flow.nodes.iter().filter(|n| proposed.node(&n.id).is_none()).map(|n| n.name.as_str()).collect();
    let mut out =
        json!({"summary": diff.summary, "notes": diff.notes, "changed_steps": changed, "removed_steps": removed});
    if arguments.get("apply").and_then(Value::as_bool).unwrap_or(false) {
        out["saved"] = save(ctx, proposed).await?;
    } else {
        out["next"] = json!("Call refine_flow again with apply: true to save this change.");
    }
    Ok(out)
}

async fn set_active(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let on = arguments.get("active").and_then(Value::as_bool).ok_or("active must be true or false")?;
    if on {
        let triggers = ctx.activation.activate(&flow.id).await.map_err(|e| e.to_string())?;
        Ok(json!({"flow_id": flow.id, "active": true, "triggers": triggers}))
    } else {
        ctx.activation.deactivate(&flow.id).await.map_err(|e| e.to_string())?;
        Ok(json!({"flow_id": flow.id, "active": false}))
    }
}

async fn integration_guide(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    if flow.kind != FlowKind::App || !ctx.services.store.is_flow_active(&flow.id).await.map_err(|e| e.to_string())? {
        return Err("the guide is for apps that are on; turn it on with set_flow_active".into());
    }
    Ok(Value::String(guide::generate(&flow, base_url(ctx))))
}

fn base_url(ctx: &RouteCtx) -> &str {
    ctx.config.pointer("/server/public_url").and_then(Value::as_str).unwrap_or("http://127.0.0.1:8080")
}

async fn export_flow(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let entries = export::bundle(&ctx.services, &ctx.config, &flow).await.map_err(|e| e.to_string())?;
    let manifest: export::Manifest = entries
        .iter()
        .find(|e| e.path == export::manifest::PATH)
        .and_then(|e| serde_json::from_slice(&e.bytes).ok())
        .ok_or("the bundle has no manifest")?;
    let (zip, folder) = (export::file_name(&flow), slug(&flow));
    let url = format!("{}/mcp/flows/{}/export", base_url(ctx).trim_end_matches('/'), flow.id);
    let download = format!("curl -fsSL -H \"Authorization: Bearer $INTERLOCK_KEY\" {url} -o {zip}");
    Ok(json!({
        "command": format!("{download} && unzip -q {zip} && rm {zip}"),
        "command_windows": format!("{}; tar -xf {zip}; del {zip}", download.replacen("curl", "curl.exe", 1)),
        "folder": folder,
        "files": entries.iter().map(|e| e.path.as_str()).collect::<Vec<_>>(),
        "settings": manifest.env.iter().map(|v| json!({"name": v.name, "required": v.required, "about": v.about}))
            .collect::<Vec<_>>(),
        "next": format!("Run the command where the folder `{folder}/` should go, then in it: copy .env.example to             .env and fill it in (never commit .env), and `docker compose up -d`. {folder}/INTERLOCK.md explains             how to call and change it."),
    }))
}
