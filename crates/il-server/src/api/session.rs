//! Sessions and the workspace extractor. A cookie `il_session` maps to a `Session` stored in the
//! store's key/value table (`ws:session:<token>`). Without a session, requests act on the local
//! workspace (`ws_local`) when `server.local_workspace` is on, so a self-hosted single user needs no
//! sign-in. Signing in (WS-1) gives each person their own workspace.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, header};
use il_core::ApiError;
use il_core::dto::UserSummary;
use serde::{Deserialize, Serialize};

use super::ApiErr;
use crate::wiring::AppState;

/// Session cookie name.
pub const SESSION_COOKIE: &str = "il_session";
/// Workspace used when no session exists.
const LOCAL_WORKSPACE: &str = "ws_local";

/// What a session token stands for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub workspace_id: String,
    pub user: Option<UserSummary>,
}

/// The caller's workspace (from the session cookie, or the local workspace).
#[derive(Debug, Clone)]
pub struct Ws(pub Session);

impl Ws {
    /// Workspace id.
    pub fn id(&self) -> &str {
        &self.0.workspace_id
    }
}

/// Store key for a session token.
pub fn session_key(token: &str) -> String {
    format!("ws:session:{token}")
}

/// A cookie's value from the request headers.
pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
}

/// A `Set-Cookie` value: http-only, `SameSite=Lax`, and `Secure` when the public URL is https.
pub fn set_cookie(state: &AppState, name: &str, value: &str, max_age: u32, path: &str) -> String {
    let secure = if state.config.server.public_url.as_deref().is_some_and(|url| url.starts_with("https://")) {
        "; Secure"
    } else {
        ""
    };
    format!("{name}={value}; Path={path}; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}

impl FromRequestParts<AppState> for Ws {
    type Rejection = ApiErr;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if let Some(token) = cookie_value(&parts.headers, SESSION_COOKIE) {
            let value = state.services.store.kv_get(&session_key(&token)).await.map_err(ApiErr::internal)?;
            if let Some(session) = value.and_then(|value| serde_json::from_value::<Session>(value).ok()) {
                return Ok(Ws(session));
            }
            return Err(ApiErr(
                StatusCode::UNAUTHORIZED,
                ApiError::new("session_expired", "This session has expired."),
            ));
        }
        if !state.config.server.local_workspace {
            return Err(ApiErr(StatusCode::UNAUTHORIZED, ApiError::new("sign_in_required", "Sign in to continue.")));
        }
        Ok(Ws(Session { workspace_id: LOCAL_WORKSPACE.into(), user: None }))
    }
}
