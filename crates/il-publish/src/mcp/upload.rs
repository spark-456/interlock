//! `POST /mcp/files`: stores one file for the API key's workspace, so a coding agent can attach
//! local documents and voice memos to `create_flow_from_goal`. MCP tool calls carry only JSON, and a
//! PDF sent as base64 through the agent's context would cost tens of thousands of tokens.

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use il_sdk::registry::RouteCtx;
use serde_json::json;

/// Largest upload, as for files added in the app.
pub(super) const MAX_UPLOAD: usize = 50 * 1024 * 1024;
const NAME_HEADER: &str = "x-file-name";

/// The command the build tool's description shows agents.
pub(super) const COMMAND: &str = "curl -H \"Authorization: Bearer $INTERLOCK_KEY\" -H \"X-File-Name: <name>\" \
     --data-binary @<path> <this server>/mcp/files";

pub(super) async fn upload(State(ctx): State<RouteCtx>, headers: HeaderMap, body: Bytes) -> Response {
    let workspace_id = match super::workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };
    let name = headers.get(NAME_HEADER).and_then(|v| v.to_str().ok()).map(str::trim).unwrap_or_default();
    if name.is_empty() || body.is_empty() {
        let error = "Send the file as the request body, with its name in an X-File-Name header.";
        return (StatusCode::BAD_REQUEST, Json(json!({"error": error}))).into_response();
    }
    let mime = mime_guess::from_path(name).first_or_octet_stream().to_string();
    match ctx.services.files.put_for_workspace(&workspace_id, name, &mime, body).await {
        Ok(file) => (StatusCode::CREATED, Json(json!({"file_id": file.id, "name": file.name, "mime": file.mime})))
            .into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))).into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use il_sdk::registry::RouteCtx;
    use il_sdk::services::{NoActivation, NoFlowTools};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::keys::KeyManager;

    #[tokio::test]
    async fn q31_files_upload_to_the_keys_workspace() {
        let fakes = FakeServices::default();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(NoFlowTools),
            activation: Arc::new(NoActivation),
            config: json!({}),
        };
        let router = super::super::routes(&ctx);
        let (_, key) = KeyManager::new(fakes.store.clone()).create("ws_a", "test").await.unwrap();
        let send = |auth: Option<String>, name: Option<&str>| {
            let mut req = Request::post("/mcp/files");
            if let Some(auth) = auth {
                req = req.header("authorization", format!("Bearer {auth}"));
            }
            if let Some(name) = name {
                req = req.header("x-file-name", name);
            }
            router.clone().oneshot(req.body(Body::from("%PDF-1.4")).unwrap())
        };
        assert_eq!(send(None, Some("faq.pdf")).await.unwrap().status(), StatusCode::UNAUTHORIZED);
        assert_eq!(send(Some(key.clone()), None).await.unwrap().status(), StatusCode::BAD_REQUEST);
        let created = send(Some(key), Some("faq.pdf")).await.unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let body = axum::body::to_bytes(created.into_body(), 1 << 16).await.unwrap();
        let file: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(file["mime"], "application/pdf");
        let id = file["file_id"].as_str().unwrap();
        assert_eq!(ctx.services.files.owner(id).await.unwrap().as_deref(), Some("ws_a"));
    }
}
