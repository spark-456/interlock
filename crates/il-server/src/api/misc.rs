//! Me, demo sessions, files, knowledge.

use axum::body::Body;
use axum::extract::{DefaultBodyLimit, Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::FileRef;
use il_core::dto::Me;
use il_sdk::services::knowledge::SourceInput;
use serde::Deserialize;
use serde_json::{Value, json};

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
        .route("/api/knowledge", get(list_knowledge).post(add_knowledge))
}

async fn me(ws: Ws) -> Json<Me> {
    Json(Me { user: ws.0.user.clone(), workspace_id: ws.0.workspace_id.clone(), demo: ws.0.demo })
}

/// WS-2: a fresh demo workspace per browser, as an http-only cookie.
async fn demo_session(State(state): State<AppState>) -> ApiResult<Response> {
    let token = format!("{}{}", il_sdk::id("ses"), il_sdk::id("")).replace('_', "");
    let session = Session { workspace_id: il_sdk::id("ws_demo"), user: None, demo: true };
    state
        .services
        .store
        .register_demo_workspace(&session.workspace_id, state.services.clock.now())
        .await
        .map_err(ApiErr::internal)?;
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
    ws: Ws,
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
        out.push(state.services.files.put_for_workspace(ws.id(), &name, &mime, bytes).await.map_err(ApiErr::internal)?);
    }
    if out.is_empty() {
        return Err(ApiErr::bad_request("Attach at least one file."));
    }
    Ok((StatusCode::CREATED, Json(out)))
}

async fn download(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Response> {
    // Files saved for a workspace are only served to it; step outputs and fixtures have no owner.
    let owner = state.services.files.owner(&id).await.map_err(|_| ApiErr::not_found("File"))?;
    if owner.is_some_and(|owner| owner != ws.id()) {
        return Err(ApiErr::not_found("File"));
    }
    let (file, bytes) = state.services.files.get(&id).await.map_err(|_| ApiErr::not_found("File"))?;
    let disposition = format!("inline; filename=\"{}\"", file.name.replace('"', ""));
    Ok(([(header::CONTENT_TYPE, file.mime), (header::CONTENT_DISPOSITION, disposition)], Body::from(bytes))
        .into_response())
}

/// `GET /api/knowledge`: the workspace's knowledge bases with their source counts.
async fn list_knowledge(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<Vec<Value>>> {
    state.services.knowledge.list_bases(ws.id()).await.map(Json).map_err(ApiErr::internal)
}

/// `POST /api/knowledge` body: a new base (`name`) or more sources for an existing one (`knowledgeBase`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddKnowledge {
    knowledge_base: Option<String>,
    name: Option<String>,
    #[serde(default)]
    sources: Vec<SourceInput>,
}

async fn add_knowledge(
    State(state): State<AppState>,
    ws: Ws,
    Json(req): Json<AddKnowledge>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let knowledge = &state.services.knowledge;
    let id = match (req.knowledge_base, req.name) {
        (Some(id), _) => {
            if !knowledge.owns_base(ws.id(), &id).await.map_err(ApiErr::internal)? {
                return Err(ApiErr::not_found("Knowledge base"));
            }
            id
        }
        (None, Some(name)) if !name.trim().is_empty() => {
            knowledge.create_base(ws.id(), name.trim()).await.map_err(ApiErr::internal)?
        }
        _ => return Err(ApiErr::bad_request("give a name for a new knowledge base")),
    };
    let sources = if req.sources.is_empty() {
        vec![]
    } else {
        knowledge.add_sources(&id, req.sources).await.map_err(|e| ApiErr::bad_request(e.to_string()))?
    };
    Ok((StatusCode::CREATED, Json(json!({ "knowledgeBase": id, "sources": sources }))))
}
