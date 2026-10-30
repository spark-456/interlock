//! Connections: the workspace's stored credentials (never their secrets), and saving key-based ones.

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use il_core::dto::{ConnectionStatus, ConnectionSummary, CreateConnectionRequest};
use il_sdk::services::store::{Credential, CredentialInfo};
use serde_json::json;

use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/connections", get(list).post(create))
}

fn summary(c: CredentialInfo) -> ConnectionSummary {
    let text = |key: &str| c.meta[key].as_str().map(String::from);
    // OAuth credentials refresh themselves, so every stored credential counts as connected.
    ConnectionSummary {
        label: text("label").unwrap_or_else(|| c.app.clone()),
        account: text("account"),
        status: ConnectionStatus::Connected,
        app: c.app,
        id: Some(c.id),
    }
}

async fn list(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<Vec<ConnectionSummary>>> {
    let creds = state.services.store.credentials(&ws.0.workspace_id).await.map_err(ApiErr::internal)?;
    Ok(Json(creds.into_iter().map(summary).collect()))
}

async fn create(
    State(state): State<AppState>,
    ws: Ws,
    Json(req): Json<CreateConnectionRequest>,
) -> ApiResult<(StatusCode, Json<ConnectionSummary>)> {
    let app = req.app.trim();
    if app.is_empty() || req.secret.is_empty() || req.secret.values().any(|v| v.trim().is_empty()) {
        return Err(ApiErr::bad_request("Name the app and fill in every field."));
    }
    let label = req.label.as_deref().map(str::trim).filter(|l| !l.is_empty()).unwrap_or(app);
    let credential = Credential {
        id: il_sdk::id("cred"),
        app: app.into(),
        secret: json!(req.secret),
        meta: json!({ "label": label, "kind": "key" }),
    };
    state.services.store.put_credential(&ws.0.workspace_id, &credential).await.map_err(ApiErr::internal)?;
    let info = CredentialInfo { id: credential.id, app: credential.app, meta: credential.meta };
    Ok((StatusCode::CREATED, Json(summary(info))))
}
