//! Sign-in with Google or GitHub (WS-1): OAuth with a one-use, browser-bound state token. Each
//! person gets their own workspace, keyed by the provider's stable id.

mod provider;
#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::{Json, Router};
use il_core::ApiError;
use il_core::dto::AuthProvider;
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use super::session::{SESSION_COOKIE, Session, cookie_value, session_key, set_cookie};
use super::{ApiErr, ApiResult};
use crate::wiring::AppState;
use provider::Provider;

const STATE_COOKIE: &str = "il_oauth_state";
/// Sessions last a week.
const SESSION_SECS: u32 = 7 * 24 * 3600;

#[derive(Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/auth/providers", get(providers))
        .route("/api/auth/{provider}/start", get(start))
        .route("/api/auth/{provider}/callback", get(callback))
}

/// The providers this server can sign people in with.
async fn providers(State(state): State<AppState>) -> Json<Vec<AuthProvider>> {
    Json(
        Provider::ALL
            .into_iter()
            .filter(|p| p.configured(&state))
            .map(|p| AuthProvider { id: p.id().into(), label: p.label().into() })
            .collect(),
    )
}

fn provider(id: &str) -> ApiResult<Provider> {
    Provider::ALL.into_iter().find(|p| p.id() == id).ok_or_else(|| ApiErr::not_found("Sign-in provider"))
}

fn callback_url(state: &AppState, provider: Provider) -> String {
    let fallback = format!("http://127.0.0.1:{}", state.config.server.port);
    let origin = state.config.server.public_url.as_deref().unwrap_or(&fallback);
    format!("{}/api/auth/{}/callback", origin.trim_end_matches('/'), provider.id())
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn state_key(provider: Provider, token: &str) -> String {
    format!("auth:{}:state:{token}", provider.id())
}

fn cookie_path(provider: Provider) -> String {
    format!("/api/auth/{}", provider.id())
}

fn unconfigured(provider: Provider) -> ApiErr {
    let message = format!("{} sign-in is not configured.", provider.label());
    ApiErr(StatusCode::SERVICE_UNAVAILABLE, ApiError::new(&format!("{}_unconfigured", provider.id()), &message))
}

fn upstream(provider: Provider) -> ApiErr {
    let message = format!("{} sign-in failed. Try again.", provider.label());
    ApiErr(StatusCode::BAD_GATEWAY, ApiError::new(&format!("{}_unavailable", provider.id()), &message))
}

async fn start(State(state): State<AppState>, Path(id): Path<String>) -> ApiResult<Response> {
    let provider = provider(&id)?;
    let client_id =
        provider.client_id(&state).filter(|_| provider.configured(&state)).ok_or_else(|| unconfigured(provider))?;
    let mut url = Url::parse(provider.authorize_url(&state)).map_err(ApiErr::internal)?;
    let token = il_sdk::id("oauth");
    let saved = json!({"expiresAt": now() + 600});
    state.services.store.kv_put(&state_key(provider, &token), &saved).await.map_err(ApiErr::internal)?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &callback_url(&state, provider))
        .append_pair("state", &token)
        .extend_pairs(provider.authorize_params());
    let mut response = Redirect::temporary(url.as_str()).into_response();
    let cookie = set_cookie(&state, STATE_COOKIE, &token, 600, &cookie_path(provider));
    response.headers_mut().insert(header::SET_COOKIE, HeaderValue::from_str(&cookie).map_err(ApiErr::internal)?);
    Ok(response)
}

/// The provider sends the browser here. A failure goes back to the sign-in page with the reason,
/// rather than showing the browser an error body.
async fn callback(
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> Response {
    match finish(state, &id, headers, query).await {
        Ok(response) => response,
        Err(ApiErr(_, error)) => {
            let query = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("error", &error.error.message)
                .finish();
            Redirect::to(&format!("/signin?{query}")).into_response()
        }
    }
}

async fn finish(state: AppState, id: &str, headers: HeaderMap, query: Callback) -> ApiResult<Response> {
    let provider = provider(id)?;
    if query.error.is_some() {
        return Err(ApiErr::bad_request(format!("{} sign-in was cancelled.", provider.label())));
    }
    let code = query.code.as_deref().filter(|s| !s.is_empty()).ok_or_else(|| ApiErr::bad_request("Missing code."))?;
    let token =
        query.state.as_deref().filter(|s| !s.is_empty()).ok_or_else(|| ApiErr::bad_request("Missing OAuth state."))?;
    if cookie_value(&headers, STATE_COOKIE).as_deref() != Some(token) {
        return Err(ApiErr::bad_request("OAuth state mismatch."));
    }
    let key = state_key(provider, token);
    let saved = state.services.store.kv_get(&key).await.map_err(ApiErr::internal)?;
    let expires = saved.as_ref().and_then(|v| v.get("expiresAt")).and_then(Value::as_u64).unwrap_or(0);
    if expires < now() {
        return Err(ApiErr::bad_request("OAuth state expired."));
    }
    state.services.store.kv_put(&key, &Value::Null).await.map_err(ApiErr::internal)?;
    if !provider.configured(&state) {
        return Err(unconfigured(provider));
    }
    let access =
        provider.access_token(&state, code, &callback_url(&state, provider)).await.map_err(|_| upstream(provider))?;
    let profile = provider.profile(&state, &access).await.map_err(|_| upstream(provider))?;
    let session = Session { workspace_id: profile.workspace_id, user: Some(profile.user) };
    let session_token = il_sdk::id("ses");
    let value = serde_json::to_value(session).map_err(ApiErr::internal)?;
    state.services.store.kv_put(&session_key(&session_token), &value).await.map_err(ApiErr::internal)?;
    let mut response = Redirect::to("/flows").into_response();
    for value in [
        set_cookie(&state, SESSION_COOKIE, &session_token, SESSION_SECS, "/"),
        set_cookie(&state, STATE_COOKIE, "", 0, &cookie_path(provider)),
    ] {
        response.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&value).map_err(ApiErr::internal)?);
    }
    Ok(response)
}
