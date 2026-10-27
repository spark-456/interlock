//! Session-scoped API key management routes.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get};
use axum::{Json, Router};
use il_core::ApiError;
use il_core::dto::{ApiKeySummary, CreateApiKeyRequest, CreateApiKeyResponse};
use il_sdk::registry::{RouteCtx, RouteFactory};

use super::{KeyError, KeyManager};

type RouteError = (StatusCode, Json<ApiError>);

fn error(status: StatusCode, code: &str, message: &str) -> RouteError {
    (status, Json(ApiError::new(code, message)))
}

fn cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(name, _)| *name == "il_session")
        .map(|(_, token)| token)
}

async fn workspace(ctx: &RouteCtx, headers: &HeaderMap) -> Result<String, RouteError> {
    let token = cookie(headers).ok_or_else(|| error(StatusCode::UNAUTHORIZED, "unauthorized", "Sign in first."))?;
    let value = ctx
        .services
        .store
        .kv_get(&format!("ws:session:{token}"))
        .await
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "Could not read the session."))?
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "unauthorized", "Sign in again."))?;
    if value["demo"] == true {
        return Err(error(StatusCode::FORBIDDEN, "demo_restricted", "API keys are unavailable in demo mode."));
    }
    value["workspaceId"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "unauthorized", "Sign in again."))
}

fn key_error(error_value: KeyError) -> RouteError {
    match error_value {
        KeyError::NotFound => error(StatusCode::NOT_FOUND, "not_found", "API key not found."),
        KeyError::Store(_) => error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "Could not update API keys."),
    }
}

/// Routes for creating, listing and revoking API keys.
pub fn routes(ctx: &RouteCtx) -> Router {
    Router::new()
        .route("/api/keys", get(list).post(create))
        .route("/api/keys/{id}", delete(revoke))
        .with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "publish.keys", make: routes } }

async fn list(State(ctx): State<RouteCtx>, headers: HeaderMap) -> Result<Json<Vec<ApiKeySummary>>, RouteError> {
    let workspace_id = workspace(&ctx, &headers).await?;
    let keys = KeyManager::new(ctx.services.store.clone()).list(&workspace_id).await.map_err(key_error)?;
    Ok(Json(keys))
}

async fn create(
    State(ctx): State<RouteCtx>,
    headers: HeaderMap,
    Json(input): Json<CreateApiKeyRequest>,
) -> Result<Response, RouteError> {
    let workspace_id = workspace(&ctx, &headers).await?;
    let name = input.name.trim();
    if name.is_empty() || name.len() > 64 {
        return Err(error(StatusCode::BAD_REQUEST, "bad_request", "Key name must be 1–64 characters."));
    }
    let (id, key) = KeyManager::new(ctx.services.store.clone()).create(&workspace_id, name).await.map_err(key_error)?;
    Ok((StatusCode::CREATED, Json(CreateApiKeyResponse { id, key })).into_response())
}

async fn revoke(
    State(ctx): State<RouteCtx>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, RouteError> {
    let workspace_id = workspace(&ctx, &headers).await?;
    KeyManager::new(ctx.services.store.clone()).revoke(&workspace_id, &id).await.map_err(key_error)?;
    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::{Body, to_bytes};
    use axum::http::Request;
    use il_sdk::services::Store;
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn p01_key_routes_reveal_once_and_track_last_use() {
        let fakes = FakeServices::default();
        fakes.store.kv_put("ws:session:local", &json!({"workspaceId":"ws_a","demo":false})).await.unwrap();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            config: Value::Null,
        };
        let router = routes(&ctx);
        let input = json!({"name":"Test"}).to_string();
        let denied = router
            .clone()
            .oneshot(
                Request::post("/api/keys")
                    .header("content-type", "application/json")
                    .body(Body::from(input.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let created = router
            .clone()
            .oneshot(
                Request::post("/api/keys")
                    .header("content-type", "application/json")
                    .header(header::COOKIE, "il_session=local")
                    .body(Body::from(input))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(created.status(), StatusCode::CREATED);
        let created: Value = serde_json::from_slice(&to_bytes(created.into_body(), 4096).await.unwrap()).unwrap();
        let id = created["id"].as_str().unwrap();
        let key = created["key"].as_str().unwrap();
        let manager = KeyManager::new(fakes.store);
        assert_eq!(manager.authenticate(key).await.unwrap().as_deref(), Some("ws_a"));
        let listed = router
            .clone()
            .oneshot(Request::get("/api/keys").header(header::COOKIE, "il_session=local").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = to_bytes(listed.into_body(), 4096).await.unwrap();
        let list: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(list[0]["id"], id);
        assert!(list[0]["lastUsedAt"].is_string());
        assert!(!String::from_utf8_lossy(&body).contains(key));
        let revoked = router
            .oneshot(
                Request::delete(format!("/api/keys/{id}"))
                    .header(header::COOKIE, "il_session=local")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(revoked.status(), StatusCode::NO_CONTENT);
        assert!(manager.authenticate(key).await.unwrap().is_none());
    }
}
