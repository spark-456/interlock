//! Platform tools for an authenticated MCP client (Claude Code, Codex…): everything a person does in
//! the app — build from a goal and answer its questions, read, edit and refine flows, test them, turn
//! them on, follow runs and decide approvals — limited to the key's workspace.

mod builds;
mod flows;
mod runs;

use il_core::Flow;
use il_sdk::registry::RouteCtx;
use rmcp::model::{JsonObject, Tool};
use serde_json::Value;

pub(super) fn tools() -> Vec<Tool> {
    [flows::tools(), builds::tools(), runs::tools()].concat()
}

pub(super) async fn call(
    ctx: &RouteCtx,
    workspace_id: &str,
    name: &str,
    arguments: &JsonObject,
) -> Result<Value, String> {
    if let Some(result) = flows::call(ctx, workspace_id, name, arguments).await {
        return result;
    }
    if let Some(result) = builds::call(ctx, workspace_id, name, arguments).await {
        return result;
    }
    runs::call(ctx, workspace_id, name, arguments).await.unwrap_or_else(|| Err("tool not found".into()))
}

fn string_arg(arguments: &JsonObject, key: &str) -> Result<String, String> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| format!("{key} is required"))
}

fn strings_arg(arguments: &JsonObject, key: &str) -> Vec<String> {
    arguments
        .get(key)
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_owned).collect())
        .unwrap_or_default()
}

/// The workspace's flow `id`; other workspaces' flows are "not found".
async fn owned_flow(ctx: &RouteCtx, workspace_id: &str, id: &str) -> Result<Flow, String> {
    ctx.services
        .store
        .get_flow(id)
        .await
        .map_err(|e| e.to_string())?
        .filter(|flow| flow.workspace_id == workspace_id)
        .ok_or_else(|| "flow not found".into())
}

/// `text` cut to `max` characters, for previews that keep replies small.
fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max { text.into() } else { format!("{}…", text.chars().take(max).collect::<String>()) }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use il_core::Flow;
    use il_sdk::registry::RouteCtx;
    use il_sdk::services::{NoActivation, NoFlowTools, Store};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use rmcp::model::JsonObject;
    use serde_json::{Value, json};

    fn args(v: Value) -> JsonObject {
        v.as_object().cloned().unwrap_or_default()
    }

    async fn setup() -> (RouteCtx, Arc<FakeRunner>) {
        let fakes = FakeServices::default();
        let runner = Arc::new(FakeRunner::default());
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: runner.clone(),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(NoFlowTools),
            activation: Arc::new(NoActivation),
            config: json!({}),
        };
        for (id, ws) in [("flw_a", "ws_a"), ("flw_b", "ws_b")] {
            let flow: Flow = serde_json::from_value(json!({"id": id, "workspaceId": ws, "version": 1, "name": "Help",
                "kind": "app", "nodes": [{"id": "n_chat", "type": "trigger.chat", "name": "Chat", "settings": {}}],
                "edges": []}))
            .unwrap();
            fakes.store.put_flow(&flow).await.unwrap();
        }
        (ctx, runner)
    }

    #[tokio::test]
    async fn q23_platform_tools_cover_the_app_and_stay_in_the_workspace() {
        let (ctx, runner) = setup().await;
        let names: Vec<String> = super::tools().iter().map(|t| t.name.to_string()).collect();
        for name in [
            "get_flow",
            "update_flow",
            "refine_flow",
            "get_build",
            "answer_build_questions",
            "test_flow",
            "get_run",
            "set_flow_active",
            "decide_approval",
            "list_step_types",
        ] {
            assert!(names.iter().any(|n| n == name), "{name} missing");
        }

        let flow = super::call(&ctx, "ws_a", "get_flow", &args(json!({"flow_id": "flw_a"}))).await.unwrap();
        assert_eq!(flow["nodes"][0]["id"], "n_chat");
        let other = super::call(&ctx, "ws_a", "get_flow", &args(json!({"flow_id": "flw_b"}))).await;
        assert_eq!(other.unwrap_err(), "flow not found", "another workspace's flow is hidden");

        let tested =
            super::call(&ctx, "ws_a", "test_flow", &args(json!({"flow_id": "flw_a", "message": "Hi"}))).await.unwrap();
        assert_eq!(runner.started()[0].trigger, "test", "test_flow never runs live");
        assert_eq!(tested["reply"][0]["message"], "Hi");

        let mut edited = flow.clone();
        edited["name"] = json!("Help desk");
        let saved =
            super::call(&ctx, "ws_a", "update_flow", &args(json!({"flow_id": "flw_a", "flow": edited}))).await.unwrap();
        assert_eq!(saved["version"], 2);
        let refined =
            super::call(&ctx, "ws_a", "refine_flow", &args(json!({"flow_id": "flw_a", "request": "be warmer"})))
                .await
                .unwrap();
        assert_eq!(refined["summary"], "no changes");
        assert!(refined.get("saved").is_none(), "refine only proposes unless asked to apply");

        let build =
            super::call(&ctx, "ws_a", "create_flow_from_goal", &args(json!({"goal": "triage email"}))).await.unwrap();
        let id = build["build_id"].as_str().unwrap();
        let status = super::call(&ctx, "ws_a", "get_build", &args(json!({"build_id": id}))).await.unwrap();
        assert_eq!(status["status"], "building");
        let foreign = super::call(&ctx, "ws_b", "get_build", &args(json!({"build_id": id}))).await;
        assert_eq!(foreign.unwrap_err(), "build not found");

        let steps = super::call(&ctx, "ws_a", "list_step_types", &args(json!({"query": "zzz-none"}))).await.unwrap();
        assert_eq!(steps, json!([]));
        assert_eq!(super::call(&ctx, "ws_a", "nope", &args(json!({}))).await.unwrap_err(), "tool not found");
    }
}
