//! Internal HTTP API (PRD §8.5). One file per area; all errors use the PRD error shape.

mod builds;
mod flows;
mod github;
mod misc;
mod runs;
mod session;
mod sse;

use axum::Json;
use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use il_core::ApiError;

pub use session::Ws;

use crate::wiring::AppState;

/// All `/api` routes.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(flows::routes())
        .merge(runs::routes())
        .merge(builds::routes())
        .merge(misc::routes())
        .merge(github::routes())
}

/// An API error response: status + `{ error: { code, message, details } }`.
#[derive(Debug)]
pub struct ApiErr(pub StatusCode, pub ApiError);

impl ApiErr {
    /// 404.
    pub fn not_found(what: &str) -> Self {
        Self(StatusCode::NOT_FOUND, ApiError::new("not_found", format!("{what} not found.")))
    }
    /// 400.
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self(StatusCode::BAD_REQUEST, ApiError::new("bad_request", message))
    }
    /// 409.
    pub fn conflict(code: &str, message: impl Into<String>) -> Self {
        Self(StatusCode::CONFLICT, ApiError::new(code, message))
    }
    /// 501 for endpoints whose owning task has not landed yet.
    pub fn not_implemented(task: &str) -> Self {
        Self(
            StatusCode::NOT_IMPLEMENTED,
            ApiError::new("not_implemented", format!("This endpoint arrives with task {task}.")),
        )
    }
    /// 500 with a logged cause; the client gets a generic sentence.
    pub fn internal(e: impl std::fmt::Display) -> Self {
        tracing::error!(error = %e, "internal error");
        Self(StatusCode::INTERNAL_SERVER_ERROR, ApiError::new("internal", "Something went wrong on our side."))
    }
}

impl IntoResponse for ApiErr {
    fn into_response(self) -> Response {
        (self.0, Json(self.1)).into_response()
    }
}

/// Result alias for handlers.
pub type ApiResult<T> = Result<T, ApiErr>;
