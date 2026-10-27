//! Flows: CRUD, versions, activate/deactivate, test runs, Ask Interlock.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::dto::{AskDiff, AskRequest, FlowSummary, RunCreated, RunSummary, TestRunRequest};
use il_core::{Flow, Item, Run};
use il_sdk::services::StartRun;
use serde_json::{Value, json};

use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/flows", get(list).post(create))
        .route("/api/flows/{id}", get(read).put(update).delete(remove))
        .route("/api/flows/{id}/versions", get(versions))
        .route("/api/flows/{id}/guide", get(guide))
        .route("/api/flows/{id}/activate", post(activate))
        .route("/api/flows/{id}/deactivate", post(deactivate))
        .route("/api/flows/{id}/test", post(test))
        .route("/api/flows/{id}/ask", post(ask))
        .route("/api/flows/{id}/nodes/{node_id}/run", post(run_node))
}

/// Loads a flow the caller's workspace owns (404 otherwise, so ids don't leak).
pub(super) async fn owned(state: &AppState, ws: &Ws, id: &str) -> ApiResult<Flow> {
    match state.services.store.get_flow(id).await.map_err(ApiErr::internal)? {
        Some(f) if f.workspace_id == ws.id() => Ok(f),
        _ => Err(ApiErr::not_found("Flow")),
    }
}

/// Short run row.
pub(super) fn summary(r: &Run) -> RunSummary {
    RunSummary {
        id: r.id.clone(),
        status: r.status,
        started_at: r.started_at,
        ms: r.finished_at.map(|f| (f - r.started_at).num_milliseconds().max(0) as u64),
        cost_usd: r.cost_usd,
        replay: r.replay,
    }
}

async fn list(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<Vec<FlowSummary>>> {
    let store = &state.services.store;
    let flows = store.list_flows(ws.id()).await.map_err(ApiErr::internal)?;
    let mut out = Vec::with_capacity(flows.len());
    for f in flows {
        let last = store.list_runs(&f.id, 1).await.ok().and_then(|r| r.into_iter().next());
        let mut apps: Vec<String> = f
            .nodes
            .iter()
            .filter_map(|n| il_sdk::registry::node(&n.node_type).and_then(|e| e.manifest().app.clone()))
            .collect();
        apps.sort();
        apps.dedup();
        out.push(FlowSummary {
            active: store.is_flow_active(&f.id).await.unwrap_or(false),
            updated_at: last.as_ref().map(|r| r.started_at).unwrap_or_else(|| state.services.clock.now()),
            last_run: last.as_ref().map(summary),
            id: f.id,
            name: f.name,
            kind: f.kind,
            version: f.version,
            apps,
        });
    }
    Ok(Json(out))
}

async fn create(
    State(state): State<AppState>,
    ws: Ws,
    Json(mut body): Json<Value>,
) -> ApiResult<(StatusCode, Json<Flow>)> {
    let obj = body.as_object_mut().ok_or_else(|| ApiErr::bad_request("Send the flow as a JSON object."))?;
    obj.entry("id").or_insert_with(|| json!(il_sdk::id("flw")));
    obj.insert("workspaceId".into(), json!(ws.id()));
    obj.entry("version").or_insert(json!(0));
    obj.entry("nodes").or_insert(json!([]));
    obj.entry("edges").or_insert(json!([]));
    obj.entry("kind").or_insert(json!("automation"));
    let mut flow: Flow = serde_json::from_value(body).map_err(|e| ApiErr::bad_request(format!("Invalid flow: {e}")))?;
    if state.services.store.get_flow(&flow.id).await.map_err(ApiErr::internal)?.is_some() {
        return Err(ApiErr::conflict("flow_exists", "A flow with this id already exists."));
    }
    flow.version = state.services.store.put_flow(&flow).await.map_err(ApiErr::internal)?;
    Ok((StatusCode::CREATED, Json(flow)))
}

async fn read(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Flow>> {
    Ok(Json(owned(&state, &ws, &id).await?))
}

async fn guide(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
) -> ApiResult<([(header::HeaderName, &'static str); 1], String)> {
    let flow = owned(&state, &ws, &id).await?;
    let fallback = format!("http://{}:{}", state.config.server.host, state.config.server.port);
    let base = state.config.server.public_url.as_deref().unwrap_or(&fallback);
    Ok(([(header::CONTENT_TYPE, "text/markdown; charset=utf-8")], il_publish::guide::generate(&flow, base)))
}

async fn update(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
    Json(mut flow): Json<Flow>,
) -> ApiResult<Json<Flow>> {
    owned(&state, &ws, &id).await?;
    flow.id = id;
    flow.workspace_id = ws.id().to_string();
    flow.version = state.services.store.put_flow(&flow).await.map_err(ApiErr::internal)?;
    Ok(Json(flow))
}

async fn remove(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<StatusCode> {
    owned(&state, &ws, &id).await?;
    let _ = state.triggers.deactivate(&id).await;
    state.services.store.delete_flow(&id).await.map_err(ApiErr::internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn versions(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Vec<Value>>> {
    let flow = owned(&state, &ws, &id).await?;
    let mut out = Vec::new();
    for v in (1..=flow.version).rev() {
        if let Ok(Some(f)) = state.services.store.flow_version(&id, v).await {
            out.push(json!({ "version": v, "name": f.name, "nodes": f.nodes.len() }));
        }
    }
    Ok(Json(out))
}

async fn activate(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    owned(&state, &ws, &id).await?;
    let n = state.triggers.activate(&id).await.map_err(|e| ApiErr::conflict("activation_failed", e.to_string()))?;
    Ok(Json(json!({ "active": true, "triggers": n })))
}

async fn deactivate(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    owned(&state, &ws, &id).await?;
    state.triggers.deactivate(&id).await.map_err(ApiErr::internal)?;
    Ok(Json(json!({ "active": false })))
}

async fn test(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
    body: Option<Json<TestRunRequest>>,
) -> ApiResult<(StatusCode, Json<RunCreated>)> {
    owned(&state, &ws, &id).await?;
    let items = body.map(|Json(b)| b.items).unwrap_or_default().into_iter().map(Item::json).collect();
    let req = StartRun { flow_id: id, version: None, trigger: "test".into(), items, trigger_node: None };
    let run_id = state.runner.start(req).await.map_err(|e| ApiErr::conflict("run_failed", e.to_string()))?;
    Ok((StatusCode::ACCEPTED, Json(RunCreated { run_id })))
}

async fn ask(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
    Json(req): Json<AskRequest>,
) -> ApiResult<Json<AskDiff>> {
    let flow = owned(&state, &ws, &id).await?;
    state
        .architect
        .ask(&flow, &req.message)
        .await
        .map(Json)
        .map_err(|e| ApiErr(StatusCode::BAD_GATEWAY, il_core::ApiError::new("architect_failed", e.to_string())))
}

async fn run_node() -> ApiErr {
    ApiErr::not_implemented("W03 / ED-7 (run a single step)")
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use il_core::Flow;
    use tower::ServiceExt;

    use crate::{Config, build_state, routes};

    #[tokio::test]
    async fn p01_guide_route_returns_markdown_only_for_owned_flow() {
        let state = build_state(
            Config::from_value(serde_json::json!({
                "server": {"public_url": "https://interlock.example"}
            }))
            .unwrap(),
        );
        let own: Flow = serde_json::from_value(serde_json::json!({
            "id":"flw_own","workspaceId":"ws_local","version":0,"name":"Support Agent",
            "kind":"app","nodes":[],"edges":[]
        }))
        .unwrap();
        let other: Flow = serde_json::from_value(serde_json::json!({
            "id":"flw_other","workspaceId":"ws_other","version":0,"name":"Private Agent",
            "kind":"app","nodes":[],"edges":[]
        }))
        .unwrap();
        state.services.store.put_flow(&own).await.unwrap();
        state.services.store.put_flow(&other).await.unwrap();
        let app = routes::router(state);
        let response =
            app.clone().oneshot(Request::get("/api/flows/flw_own/guide").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "text/markdown; charset=utf-8");
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let guide = String::from_utf8(body.to_vec()).unwrap();
        assert!(guide.contains("https://interlock.example/v1/chat/completions"));
        let response =
            app.oneshot(Request::get("/api/flows/flw_other/guide").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
