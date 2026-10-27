//! Me, demo sessions, files, templates, connections, knowledge.

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::FileRef;
use il_core::dto::{ConnectionSummary, Me};

use super::session::{SESSION_COOKIE, Session, session_key};
use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

/// Largest file accepted (50 MB, as the Create screen says).
const MAX_UPLOAD: usize = 50 * 1024 * 1024;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/me", get(me))
        .route("/api/demo/session", post(demo_session))
        .route("/api/files", post(upload).layer(DefaultBodyLimit::max(MAX_UPLOAD + 64 * 1024)))
        .route("/api/files/{id}", get(download))
        .route("/api/connections", get(connections))
        .route("/api/knowledge", get(knowledge).post(knowledge))
}

async fn me(ws: Ws) -> Json<Me> {
    Json(Me { user: ws.0.user.clone(), workspace_id: ws.0.workspace_id.clone(), demo: ws.0.demo })
}

/// WS-2: a fresh demo workspace per browser, as an http-only cookie.
async fn demo_session(State(state): State<AppState>) -> ApiResult<Response> {
    let token = format!("{}{}", il_sdk::id("ses"), il_sdk::id("")).replace('_', "");
    let session = Session { workspace_id: il_sdk::id("ws_demo"), user: None, demo: true };
    super::templates::seed(&state, &session.workspace_id).await?;
    let value = serde_json::to_value(&session).map_err(ApiErr::internal)?;
    state.services.store.kv_put(&session_key(&token), &value).await.map_err(ApiErr::internal)?;
    let secure = if state.config.server.public_url.as_deref().is_some_and(|url| url.starts_with("https://")) {
        "; Secure"
    } else {
        ""
    };
    let cookie = format!("{SESSION_COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800{secure}");
    let me = Me { user: None, workspace_id: session.workspace_id, demo: true };
    Ok(([(header::SET_COOKIE, cookie)], Json(me)).into_response())
}

/// Multipart upload: every file part is stored; returns their `FileRef`s.
async fn upload(
    State(state): State<AppState>,
    _ws: Ws,
    mut form: Multipart,
) -> ApiResult<(StatusCode, Json<Vec<FileRef>>)> {
    let mut out = Vec::new();
    while let Some(field) = form.next_field().await.map_err(|e| ApiErr::bad_request(e.to_string()))? {
        let Some(name) = field.file_name().map(str::to_string) else { continue };
        let mime = field
            .content_type()
            .map(str::to_string)
            .unwrap_or_else(|| mime_guess::from_path(&name).first_or_octet_stream().to_string());
        let bytes = field.bytes().await.map_err(|e| ApiErr::bad_request(e.to_string()))?;
        if bytes.len() > MAX_UPLOAD {
            return Err(ApiErr(
                StatusCode::PAYLOAD_TOO_LARGE,
                il_core::ApiError::new("too_large", "Files must be 50 MB or smaller."),
            ));
        }
        out.push(state.services.files.put(&name, &mime, bytes).await.map_err(ApiErr::internal)?);
    }
    if out.is_empty() {
        return Err(ApiErr::bad_request("Attach at least one file."));
    }
    Ok((StatusCode::CREATED, Json(out)))
}

async fn download(State(state): State<AppState>, _ws: Ws, Path(id): Path<String>) -> ApiResult<Response> {
    let (file, bytes) = state.services.files.get(&id).await.map_err(|_| ApiErr::not_found("File"))?;
    let disposition = format!("inline; filename=\"{}\"", file.name.replace('"', ""));
    Ok(([(header::CONTENT_TYPE, file.mime), (header::CONTENT_DISPOSITION, disposition)], Body::from(bytes))
        .into_response())
}

/// Connections arrive with C01/C02; until then every app is listed as not connected.
async fn connections() -> Json<Vec<ConnectionSummary>> {
    Json(vec![])
}

async fn knowledge() -> ApiErr {
    ApiErr::not_implemented("K01 (knowledge)")
}
