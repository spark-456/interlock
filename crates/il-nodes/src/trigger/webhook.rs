//! `trigger.webhook` — Webhook: activation gives the node an unguessable token; requests to
//! `POST /hooks/{flowId}/{token}` start a run (or, with "respond: lastNode", wait and return
//! the Respond output). The route group self-registers here.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use il_core::ApiError;
use il_sdk::prelude::*;
use il_sdk::registry::{RouteCtx, RouteFactory};
use il_sdk::services::StartRun;
use il_sdk::{TriggerCtx, TriggerHandle};

/// Webhook node.
#[derive(Debug, Default)]
pub struct TriggerWebhook;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.webhook",
        Category::Trigger,
        "Webhook",
        "Starts the flow when an HTTP request hits the flow's secret webhook URL.",
    )
    .icon("webhook")
    .trigger()
    .status(NodeStatus::Ready)
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "method": {
                "type": "string",
                "title": "Method",
                "enum": [
                    "POST",
                    "GET",
                    "PUT"
                ],
                "default": "POST",
                "x-widget": "select"
            },
            "respond": {
                "type": "string",
                "title": "Respond",
                "enum": [
                    "immediately",
                    "lastNode"
                ],
                "default": "immediately",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Receive a POST",
        json!({
            "method": "POST",
            "respond": "immediately"
        }),
    )
});

/// Store key holding the token of one webhook node.
pub fn token_key(flow_id: &str, node_id: &str) -> String {
    format!("tr:webhook:{flow_id}:{node_id}")
}

/// Public path for a webhook.
pub fn webhook_path(flow_id: &str, token: &str) -> String {
    format!("/hooks/{flow_id}/{token}")
}

#[async_trait]
impl NodeExecutor for TriggerWebhook {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerWebhook {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        let store = &ctx.services.store;
        let key = token_key(&ctx.flow_id, &ctx.node_id);
        let existing = store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        if existing.as_ref().and_then(Value::as_str).is_none() {
            // Two ULIDs: 160 bits, of which 160 random-ish (80 random bits each).
            let token = format!("whk{}{}", il_sdk::id(""), il_sdk::id("")).replace('_', "").to_lowercase();
            store.kv_put(&key, &json!(token)).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        }
        Ok(TriggerHandle::noop())
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerWebhook)) }

fn routes(ctx: &RouteCtx) -> Router {
    Router::new().route("/hooks/{flow_id}/{token}", post(receive).get(receive)).with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "webhooks", make: routes } }

fn not_found() -> Response {
    (StatusCode::NOT_FOUND, Json(ApiError::new("not_found", "No active webhook at this address."))).into_response()
}

fn failed(message: String) -> Response {
    (StatusCode::BAD_GATEWAY, Json(ApiError::new("run_failed", message))).into_response()
}

async fn receive(
    State(ctx): State<RouteCtx>,
    Path((flow_id, token)): Path<(String, String)>,
    headers: HeaderMap,
    body: String,
) -> Response {
    let store = &ctx.services.store;
    let Ok(Some(flow)) = store.get_flow(&flow_id).await else { return not_found() };
    if !store.is_flow_active(&flow_id).await.unwrap_or(false) {
        return not_found();
    }
    let mut matched = None;
    for node in flow.nodes.iter().filter(|n| n.node_type == MANIFEST.type_id && !n.disabled) {
        let stored = store.kv_get(&token_key(&flow_id, &node.id)).await.ok().flatten();
        if stored.as_ref().and_then(Value::as_str).is_some_and(|t| constant_time_eq(t.as_bytes(), token.as_bytes())) {
            matched = Some(node.clone());
        }
    }
    let Some(node) = matched else { return not_found() };
    let json_body: Value =
        if body.trim().is_empty() { json!({}) } else { serde_json::from_str(&body).unwrap_or_else(|_| json!(body)) };
    let content_type = headers.get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
    let item = Item::json(json!({ "body": json_body, "contentType": content_type }));
    let req = StartRun {
        flow_id: flow_id.clone(),
        version: None,
        trigger: MANIFEST.type_id.clone(),
        items: vec![item],
        trigger_node: Some(node.id.clone()),
    };
    if node.settings.get("respond").and_then(Value::as_str) == Some("lastNode") {
        match ctx.runner.invoke(req).await {
            Ok(out) => {
                let mut items: Vec<Value> = out.response.into_iter().map(|i| i.json).collect();
                let body = if items.len() == 1 { items.remove(0) } else { Value::Array(items) };
                (StatusCode::OK, Json(body)).into_response()
            }
            Err(e) => failed(e.to_string()),
        }
    } else {
        match ctx.runner.start(req).await {
            Ok(run_id) => (StatusCode::ACCEPTED, Json(json!({ "runId": run_id }))).into_response(),
            Err(e) => failed(e.to_string()),
        }
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::Request;
    use il_core::{Flow, FlowKind, Node};
    use il_sdk::services::Store;
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&TriggerWebhook);
    }

    fn flow(respond: &str) -> Flow {
        Flow {
            id: "flw_hook".into(),
            workspace_id: "ws".into(),
            version: 1,
            name: "Hook".into(),
            kind: FlowKind::Automation,
            nodes: vec![Node {
                id: "n_hook".into(),
                node_type: "trigger.webhook".into(),
                type_version: 1,
                name: "Webhook".into(),
                position: Default::default(),
                group: None,
                settings: json!({ "method": "POST", "respond": respond }),
                credentials: Default::default(),
                disabled: false,
                notes: None,
            }],
            edges: vec![],
            groups: vec![],
            settings: Default::default(),
            meta: Default::default(),
        }
    }

    async fn setup(respond: &str, active: bool) -> (Router, Arc<FakeRunner>, String) {
        let fakes = FakeServices::default();
        let runner = Arc::new(FakeRunner::default());
        let f = flow(respond);
        fakes.store.put_flow(&f).await.unwrap();
        fakes.store.set_flow_active("flw_hook", active).await.unwrap();
        let tctx = TriggerCtx {
            flow_id: "flw_hook".into(),
            workspace_id: "ws".into(),
            node_id: "n_hook".into(),
            settings: f.nodes[0].settings.clone(),
            services: fakes.services(),
            runner: runner.clone(),
            cancel: CancellationToken::new(),
        };
        TriggerWebhook.start(tctx).await.unwrap();
        let token = fakes.store.kv_get(&token_key("flw_hook", "n_hook")).await.unwrap().unwrap();
        let token = token.as_str().unwrap().to_string();
        let rctx = RouteCtx {
            services: fakes.services(),
            runner: runner.clone(),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            config: json!({}),
        };
        (routes(&rctx), runner, token)
    }

    async fn send(app: Router, path: &str, body: &str) -> (StatusCode, Value) {
        let req =
            Request::post(path).header("content-type", "application/json").body(Body::from(body.to_string())).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null))
    }

    #[tokio::test]
    async fn valid_token_starts_a_run() {
        let (app, runner, token) = setup("immediately", true).await;
        assert!(token.len() > 40, "token is long and random");
        let (status, body) = send(app, &webhook_path("flw_hook", &token), r#"{"order": 42}"#).await;
        assert_eq!(status, StatusCode::ACCEPTED);
        assert!(body["runId"].is_string());
        let started = runner.started();
        assert_eq!(started[0].items[0].json["body"]["order"], 42);
        assert_eq!(started[0].trigger_node.as_deref(), Some("n_hook"));
    }

    #[tokio::test]
    async fn wrong_token_or_inactive_flow_is_not_found() {
        let (app, runner, _) = setup("immediately", true).await;
        let (status, body) = send(app, "/hooks/flw_hook/whkwrong", "{}").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "not_found");
        assert!(runner.started().is_empty());

        let (app, runner, token) = setup("immediately", false).await;
        let (status, _) = send(app, &webhook_path("flw_hook", &token), "{}").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(runner.started().is_empty());
    }

    #[tokio::test]
    async fn respond_last_node_returns_the_response() {
        let (app, _, token) = setup("lastNode", true).await;
        let (status, body) = send(app, &webhook_path("flw_hook", &token), r#"{"q": "hi"}"#).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["body"]["q"], "hi", "FakeRunner echoes the input as the response");
    }
}
