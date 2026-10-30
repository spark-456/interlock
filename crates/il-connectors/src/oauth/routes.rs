//! HTTP entry points for registered OAuth providers.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use il_sdk::registry::{RouteCtx, RouteFactory};
use il_sdk::services::Services;
use serde::Deserialize;
use serde_json::{Value, json};

use super::Broker;

const STATE_COOKIE: &str = "il_connection_state";

#[derive(Clone)]
struct RouteState {
    services: Services,
    secure_cookies: bool,
    /// `server.local_workspace`: requests without a session act on `ws_local`.
    local_workspace: bool,
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

type RouteResult = Result<Response, (StatusCode, Json<Value>)>;

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (status, Json(json!({"error":{"code":code,"message":message}})))
}

fn routes(ctx: &RouteCtx) -> Router {
    let state = RouteState {
        services: ctx.services.clone(),
        secure_cookies: ctx
            .config
            .pointer("/server/public_url")
            .and_then(Value::as_str)
            .is_some_and(|url| url.starts_with("https://")),
        local_workspace: ctx.config.pointer("/server/local_workspace").and_then(Value::as_bool).unwrap_or(true),
    };
    Router::new()
        .route("/api/oauth/{provider}/start", get(start))
        .route("/api/oauth/{provider}/callback", get(callback))
        .route("/api/oauth/{provider}/credentials/{credential_id}/refresh", post(refresh))
        .with_state(state)
}

inventory::submit! { RouteFactory { name: "oauth", make: routes } }

fn broker(state: &RouteState, provider: &str) -> Result<Broker, (StatusCode, Json<Value>)> {
    Broker::from_env(provider, state.services.store.clone(), state.services.http.clone())
        .map_err(|_| error(StatusCode::SERVICE_UNAVAILABLE, "oauth_unavailable", "This connection is not configured."))
}

fn cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
}

fn state_cookie(state: &RouteState, value: &str, max_age: u32) -> Result<HeaderValue, (StatusCode, Json<Value>)> {
    let secure = if state.secure_cookies { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{STATE_COOKIE}={value}; Path=/api/oauth; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}"
    ))
    .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "Could not complete this connection."))
}

async fn workspace_id(state: &RouteState, headers: &HeaderMap) -> Result<String, (StatusCode, Json<Value>)> {
    let Some(token) = cookie(headers, "il_session") else {
        if state.local_workspace {
            return Ok("ws_local".into());
        }
        return Err(error(StatusCode::UNAUTHORIZED, "sign_in_required", "Sign in to connect apps."));
    };
    let session = state
        .services
        .store
        .kv_get(&format!("ws:session:{token}"))
        .await
        .map_err(|_| error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "Could not read the session."))?
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "unauthorized", "Sign in again."))?;
    if session["demo"] == true {
        return Err(error(StatusCode::FORBIDDEN, "demo_restricted", "Connections are unavailable in demo mode."));
    }
    session["workspaceId"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| error(StatusCode::UNAUTHORIZED, "unauthorized", "Sign in again."))
}

async fn start(State(state): State<RouteState>, Path(provider): Path<String>, headers: HeaderMap) -> RouteResult {
    let workspace = workspace_id(&state, &headers).await?;
    let url = broker(&state, &provider)?.start(&workspace).await.map_err(|_| {
        error(StatusCode::INTERNAL_SERVER_ERROR, "oauth_start_failed", "Could not start this connection.")
    })?;
    let token = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| error(StatusCode::INTERNAL_SERVER_ERROR, "internal", "Could not start this connection."))?;
    let mut response = Redirect::temporary(url.as_str()).into_response();
    response.headers_mut().insert(header::SET_COOKIE, state_cookie(&state, &token, 600)?);
    Ok(response)
}

async fn callback(
    State(state): State<RouteState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Query(query): Query<CallbackQuery>,
) -> RouteResult {
    if query.error.is_some() {
        return Err(error(StatusCode::BAD_REQUEST, "oauth_declined", "Authorization was declined."));
    }
    let token = query
        .state
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "invalid_state", "Missing OAuth state."))?;
    if cookie(&headers, STATE_COOKIE).as_deref() != Some(token) {
        return Err(error(StatusCode::BAD_REQUEST, "invalid_state", "OAuth state mismatch."));
    }
    let code = query
        .code
        .as_deref()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "invalid_code", "Missing authorization code."))?;
    broker(&state, &provider)?
        .callback(token, code)
        .await
        .map_err(|_| error(StatusCode::BAD_GATEWAY, "oauth_callback_failed", "Could not complete this connection."))?;
    let mut response = Redirect::to("/connections").into_response();
    response.headers_mut().insert(header::SET_COOKIE, state_cookie(&state, "", 0)?);
    Ok(response)
}

async fn refresh(
    State(state): State<RouteState>,
    Path((provider, credential_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> RouteResult {
    let workspace = workspace_id(&state, &headers).await?;
    broker(&state, &provider)?
        .refresh(&workspace, &credential_id)
        .await
        .map_err(|_| error(StatusCode::BAD_GATEWAY, "oauth_refresh_failed", "Could not refresh this connection."))?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::Request;
    use il_sdk::services::{NoFlowTools, Store};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn c01_oauth_routes_reject_demo_and_unbound_callback() {
        let fakes = FakeServices::default();
        fakes.store.kv_put("ws:session:demo", &json!({"workspaceId":"ws_demo","demo":true})).await.unwrap();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(NoFlowTools),
            activation: std::sync::Arc::new(il_sdk::services::NoActivation),
            config: json!({}),
        };
        let router = routes(&ctx);
        let demo = router
            .clone()
            .oneshot(
                Request::get("/api/oauth/google/start")
                    .header(header::COOKIE, "il_session=demo")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(demo.status(), StatusCode::FORBIDDEN);
        let callback = router
            .oneshot(Request::get("/api/oauth/google/callback?state=missing&code=code").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(callback.status(), StatusCode::BAD_REQUEST);
    }
}
