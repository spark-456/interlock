//! Test runs, live runs, run details and approvals.

use il_core::{ApprovalDecision, Decision, Item, Run};
use il_sdk::registry::RouteCtx;
use il_sdk::services::StartRun;
use rmcp::model::{JsonObject, Tool};
use serde_json::{Value, json};

use super::{clip, owned_flow, string_arg};
use crate::mcp::tool;

/// Characters of each step's output shown in run details.
const OUTPUT_PREVIEW: usize = 600;

pub(super) fn tools() -> Vec<Tool> {
    let id = json!({"type": "string", "description": "Flow id from list_flows"});
    let input = json!({"description": "The trigger's item, e.g. {\"subject\": \"Refund?\"}"});
    let message = json!({"type": "string", "description": "For apps: the person's chat message"});
    vec![
        tool(
            "test_flow",
            "Test-run a flow with sample input. Nothing is sent or posted: steps that would post show a preview. \
             Returns the reply (for apps), each step's status and output, and the cost.",
            json!({"type": "object", "properties": {"flow_id": id, "input": input, "message": message},
                "required": ["flow_id"]}),
        ),
        tool(
            "run_flow",
            "Run a flow for real with this input: steps send, post and call APIs. Use test_flow to try it first.",
            json!({"type": "object", "properties": {"flow_id": id, "input": input, "message": message},
                "required": ["flow_id"]}),
        ),
        tool(
            "get_run",
            "A run's status, error, cost and each step's output.",
            json!({"type": "object", "properties": {"run_id": {"type": "string"}}, "required": ["run_id"]}),
        ),
        tool(
            "list_runs",
            "A flow's recent runs, newest first.",
            json!({"type": "object", "properties": {"flow_id": id, "limit": {"type": "integer"}}, "required": ["flow_id"]}),
        ),
        tool(
            "list_approvals",
            "Runs waiting for a person to approve or reject a step, with what they would send.",
            json!({"type": "object", "properties": {}}),
        ),
        tool(
            "decide_approval",
            "Approve or reject a waiting step; the run continues. Optionally edit what it sends (`payload`).",
            json!({"type": "object", "properties": {"approval_id": {"type": "string"}, "approve": {"type": "boolean"},
                "comment": {"type": "string"}, "payload": {"type": "object"}}, "required": ["approval_id", "approve"]}),
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
        "test_flow" => invoke(ctx, workspace_id, arguments, "test").await,
        "run_flow" => invoke(ctx, workspace_id, arguments, "mcp").await,
        "get_run" => get_run(ctx, workspace_id, arguments).await,
        "list_runs" => list_runs(ctx, workspace_id, arguments).await,
        "list_approvals" => list_approvals(ctx, workspace_id).await,
        "decide_approval" => decide(ctx, workspace_id, arguments).await,
        _ => return None,
    })
}

async fn invoke(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject, trigger: &str) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let item = match (arguments.get("message").and_then(Value::as_str), arguments.get("input")) {
        (Some(m), _) => json!({"message": m, "messages": [{"role": "user", "content": m}]}),
        (None, Some(input)) => input.clone(),
        (None, None) => flow.nodes.iter().find_map(|n| n.settings["sampleItems"].get(0).cloned()).unwrap_or(json!({})),
    };
    let req = StartRun {
        flow_id: flow.id,
        version: None,
        trigger: trigger.into(),
        items: vec![Item::json(item)],
        trigger_node: None,
        only: None,
    };
    let outcome = ctx.runner.invoke(req).await.map_err(|e| e.to_string())?;
    let reply: Vec<Value> = outcome
        .response
        .iter()
        .map(|i| i.json.get("body").or_else(|| i.json.get("answer")).cloned().unwrap_or_else(|| i.json.clone()))
        .collect();
    let mut out = details(ctx, &outcome.run).await?;
    out["reply"] = json!(reply);
    Ok(out)
}

/// A run with each step's status and a preview of its output.
async fn details(ctx: &RouteCtx, run: &Run) -> Result<Value, String> {
    let steps: Vec<Value> = ctx
        .services
        .store
        .node_runs(&run.id)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|s| {
            let output = s
                .output
                .iter()
                .map(|(port, items)| (port.clone(), json!(items.iter().map(|i| &i.json).collect::<Vec<_>>())));
            json!({"step": s.node_id, "status": s.status, "error": s.error, "ms": s.ms,
                "output": clip(&Value::Object(output.collect()).to_string(), OUTPUT_PREVIEW)})
        })
        .collect();
    Ok(json!({"run_id": run.id, "flow_id": run.flow_id, "status": run.status, "error": run.error,
        "cost_usd": run.cost_usd, "trigger": run.trigger, "started_at": run.started_at, "steps": steps}))
}

async fn get_run(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let id = string_arg(arguments, "run_id")?;
    let run = ctx
        .services
        .store
        .get_run(&id)
        .await
        .map_err(|e| e.to_string())?
        .filter(|r| r.workspace_id == workspace_id)
        .ok_or("run not found")?;
    details(ctx, &run).await
}

async fn list_runs(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let flow = owned_flow(ctx, workspace_id, &string_arg(arguments, "flow_id")?).await?;
    let limit = arguments.get("limit").and_then(Value::as_u64).unwrap_or(10).clamp(1, 50) as usize;
    let runs = ctx.services.store.list_runs(&flow.id, limit).await.map_err(|e| e.to_string())?;
    Ok(json!(
        runs.iter()
            .map(|r| json!({"run_id": r.id, "status": r.status, "trigger": r.trigger,
        "started_at": r.started_at, "cost_usd": r.cost_usd, "error": r.error}))
            .collect::<Vec<_>>()
    ))
}

async fn list_approvals(ctx: &RouteCtx, workspace_id: &str) -> Result<Value, String> {
    let approvals = ctx.services.store.list_approvals(workspace_id, true).await.map_err(|e| e.to_string())?;
    Ok(json!(
        approvals
            .iter()
            .map(|a| json!({"approval_id": a.id, "flow_id": a.flow_id, "run_id": a.run_id,
        "title": a.title, "message": a.message, "payload": a.payload, "expires_at": a.expires_at}))
            .collect::<Vec<_>>()
    ))
}

async fn decide(ctx: &RouteCtx, workspace_id: &str, arguments: &JsonObject) -> Result<Value, String> {
    let id = string_arg(arguments, "approval_id")?;
    let owned = ctx.services.store.get_approval(&id).await.map_err(|e| e.to_string())?;
    if owned.is_none_or(|a| a.workspace_id != workspace_id) {
        return Err("approval not found".into());
    }
    let approve = arguments.get("approve").and_then(Value::as_bool).ok_or("approve must be true or false")?;
    let decision = ApprovalDecision {
        decision: if approve { Decision::Approve } else { Decision::Reject },
        payload: arguments.get("payload").cloned(),
        comment: arguments.get("comment").and_then(Value::as_str).map(str::to_owned),
    };
    ctx.runner.decide(&id, decision, Some("MCP client".into())).await.map_err(|e| e.to_string())?;
    Ok(json!({"approval_id": id, "decided": if approve { "approved" } else { "rejected" }}))
}
