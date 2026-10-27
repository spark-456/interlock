//! Runs (history, detail, live events) and approvals.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::dto::RunSummary;
use il_core::{Approval, ApprovalDecision, NodeRun, Run};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio::sync::mpsc;

use super::flows::{owned, summary};
use super::{ApiErr, ApiResult, Ws, sse};
use crate::wiring::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/flows/{id}/runs", get(list))
        .route("/api/runs/{id}", get(read))
        .route("/api/runs/{id}/events", get(events))
        .route("/api/runs/{id}/cancel", post(cancel))
        .route("/api/approvals", get(approvals))
        .route("/api/approvals/{id}/decide", post(decide))
}

async fn owned_run(state: &AppState, ws: &Ws, id: &str) -> ApiResult<Run> {
    match state.services.store.get_run(id).await.map_err(ApiErr::internal)? {
        Some(r) if r.workspace_id == ws.id() => Ok(r),
        _ => Err(ApiErr::not_found("Run")),
    }
}

async fn list(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Vec<RunSummary>>> {
    owned(&state, &ws, &id).await?;
    let runs = state.services.store.list_runs(&id, 50).await.map_err(ApiErr::internal)?;
    Ok(Json(runs.iter().map(summary).collect()))
}

/// Run detail: the run (without the engine checkpoint) and per-node inputs/outputs (EN-7).
async fn read(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Value>> {
    let mut run = owned_run(&state, &ws, &id).await?;
    run.state = Value::Null;
    let nodes: Vec<NodeRun> = state.services.store.node_runs(&id).await.map_err(ApiErr::internal)?;
    Ok(Json(json!({ "run": run, "nodeRuns": nodes })))
}

#[derive(Deserialize)]
struct After {
    after: Option<u64>,
}

/// SSE: stored events after the cursor, then live events until the run ends.
async fn events(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
    Query(q): Query<After>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    owned_run(&state, &ws, &id).await?;
    let mut last = sse::cursor(q.after, &headers);
    let (tx, rx) = mpsc::channel(256);
    // Subscribe before reading history so nothing falls in the gap; duplicates are dropped by seq.
    let mut live = state.engine.events().subscribe();
    let history = state.services.store.run_events(&id, last).await.map_err(ApiErr::internal)?;
    tokio::spawn(async move {
        let mut done = false;
        for ev in history {
            last = ev.seq;
            done |= is_terminal(&ev);
            if tx.send(sse::frame(ev.seq, &ev)).await.is_err() {
                return;
            }
        }
        while !done {
            match live.recv().await {
                Ok(ev) if ev.run_id == id && ev.seq > last => {
                    last = ev.seq;
                    done = is_terminal(&ev);
                    if tx.send(sse::frame(ev.seq, &ev)).await.is_err() {
                        return;
                    }
                }
                Ok(_) => {}
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                Err(_) => return,
            }
        }
    });
    Ok(sse::response(rx))
}

fn is_terminal(ev: &il_core::RunEvent) -> bool {
    use il_core::RunEventKind::*;
    matches!(ev.kind, RunSucceeded { .. } | RunFailed { .. } | RunCancelled {})
}

async fn cancel(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<StatusCode> {
    owned_run(&state, &ws, &id).await?;
    state.runner.cancel(&id).await.map_err(|e| ApiErr::conflict("cancel_failed", e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}

async fn approvals(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<Vec<Approval>>> {
    Ok(Json(state.services.store.list_approvals(ws.id(), true).await.map_err(ApiErr::internal)?))
}

async fn decide(
    State(state): State<AppState>,
    ws: Ws,
    Path(id): Path<String>,
    Json(decision): Json<ApprovalDecision>,
) -> ApiResult<StatusCode> {
    match state.services.store.get_approval(&id).await.map_err(ApiErr::internal)? {
        Some(a) if a.workspace_id == ws.id() => {}
        _ => return Err(ApiErr::not_found("Approval")),
    }
    let by = ws.0.user.as_ref().map(|u| u.name.clone());
    state.runner.decide(&id, decision, by).await.map_err(|e| ApiErr::conflict("decision_failed", e.to_string()))?;
    Ok(StatusCode::ACCEPTED)
}
