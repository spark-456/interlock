//! Sessions and the workspace extractor. A cookie `il_session` maps to a `Session` stored in the
//! store's key/value table (`ws:session:<token>`). Without a session, requests act on the local
//! workspace (`ws_local`), so a self-hosted single user needs no sign-in. Demo sessions get an
//! ephemeral workspace each (WS-2). GitHub sign-in (WS-1) creates sessions the same way.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use il_core::dto::UserSummary;
use serde::{Deserialize, Serialize};

use super::ApiErr;
use crate::wiring::AppState;

/// Session cookie name.
pub const SESSION_COOKIE: &str = "il_session";
/// Workspace used when no session exists.
pub const LOCAL_WORKSPACE: &str = "ws_local";

/// What a session token stands for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub workspace_id: String,
    pub user: Option<UserSummary>,
    pub demo: bool,
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

fn cookie(parts: &Parts, name: &str) -> Option<String> {
    let header = parts.headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    header.split(';').filter_map(|c| c.trim().split_once('=')).find(|(k, _)| *k == name).map(|(_, v)| v.to_string())
}

impl FromRequestParts<AppState> for Ws {
    type Rejection = ApiErr;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        if let Some(token) = cookie(parts, SESSION_COOKIE)
            && let Ok(Some(v)) = state.services.store.kv_get(&session_key(&token)).await
            && let Ok(session) = serde_json::from_value::<Session>(v)
        {
            return Ok(Ws(session));
        }
        Ok(Ws(Session { workspace_id: LOCAL_WORKSPACE.into(), user: None, demo: false }))
    }
}
