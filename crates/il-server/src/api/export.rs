//! `GET /api/flows/:id/export`: the flow as a self-hosting bundle (Q36), for the Export button.

use axum::Router;
use axum::extract::{Path, State};
use axum::response::Response;
use axum::routing::get;

use super::{ApiResult, Ws};
use crate::wiring::AppState;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/flows/{id}/export", get(export))
}

async fn export(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Response> {
    let flow = super::flows::owned(&state, &ws, &id).await?;
    Ok(il_publish::export::response(&state.services, &state.config.raw, &flow).await)
}
