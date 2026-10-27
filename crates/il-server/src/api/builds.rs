//! Builds (architect sessions): start, answer, and the build event stream.

use std::time::Duration;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::BuildEventKind;
use il_core::dto::{BuildAnswers, BuildCreated, BuildRequest};
use serde::Deserialize;
use tokio::sync::mpsc;

use super::{ApiErr, ApiResult, Ws, sse};
use crate::wiring::AppState;

/// How often the build stream checks the store for new events.
const POLL: Duration = Duration::from_millis(250);

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/builds", post(start))
        .route("/api/builds/{id}/events", get(events))
        .route("/api/builds/{id}/answers", post(answer))
}

fn architect_err(e: impl std::fmt::Display) -> ApiErr {
    ApiErr(StatusCode::BAD_GATEWAY, il_core::ApiError::new("architect_failed", e.to_string()))
}

async fn start(
    State(state): State<AppState>,
    ws: Ws,
    Json(req): Json<BuildRequest>,
) -> ApiResult<(StatusCode, Json<BuildCreated>)> {
    if req.goal.trim().is_empty() {
        return Err(ApiErr::bad_request("Describe what you want to build."));
    }
    let build_id = state.architect.start_build(ws.id(), req).await.map_err(architect_err)?;
    Ok((StatusCode::ACCEPTED, Json(BuildCreated { build_id })))
}

async fn answer(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(a): Json<BuildAnswers>,
) -> ApiResult<StatusCode> {
    state.architect.answer(&id, a).await.map_err(architect_err)?;
    Ok(StatusCode::ACCEPTED)
}

#[derive(Deserialize)]
struct After {
    after: Option<u64>,
}

/// SSE over the stored build events (the architect persists every event), until ready/failed.
async fn events(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<After>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let mut last = sse::cursor(q.after, &headers);
    let (tx, rx) = mpsc::channel(256);
    let store = state.services.store.clone();
    tokio::spawn(async move {
        loop {
            let batch = match store.build_events(&id, last).await {
                Ok(b) => b,
                Err(_) => return,
            };
            for ev in batch {
                last = ev.seq;
                let end = matches!(ev.kind, BuildEventKind::Ready { .. } | BuildEventKind::Failed { .. });
                if tx.send(sse::frame(ev.seq, &ev)).await.is_err() || end {
                    return;
                }
            }
            if tx.is_closed() {
                return;
            }
            tokio::time::sleep(POLL).await;
        }
    });
    sse::response(rx)
}
