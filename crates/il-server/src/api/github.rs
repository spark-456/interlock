//! GitHub OAuth sign-in with a one-use, browser-bound state token.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use il_core::ApiError;
use il_core::dto::UserSummary;
use serde::Deserialize;
use serde_json::{Value, json};
use url::Url;

use super::session::{SESSION_COOKIE, Session, session_key};
use super::{ApiErr, ApiResult};
use crate::wiring::AppState;

const STATE_COOKIE: &str = "il_oauth_state";

#[derive(Deserialize)]
struct Callback {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/api/auth/github/start", get(start)).route("/api/auth/github/callback", get(callback))
}

fn setting<'a>(state: &'a AppState, key: &str) -> Option<&'a str> {
    state.config.raw.get("auth")?.get("github")?.get(key)?.as_str()
}

fn callback_url(state: &AppState) -> String {
    let fallback = format!("http://127.0.0.1:{}", state.config.server.port);
    let origin = state.config.server.public_url.as_deref().unwrap_or(&fallback);
    format!("{}/api/auth/github/callback", origin.trim_end_matches('/'))
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn state_key(token: &str) -> String {
    format!("auth:github:state:{token}")
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value.to_owned())
}

fn cookie(state: &AppState, name: &str, value: &str, max_age: u32, path: &str) -> String {
    let secure = if state.config.server.public_url.as_deref().is_some_and(|url| url.starts_with("https://")) {
        "; Secure"
    } else {
        ""
    };
    format!("{name}={value}; Path={path}; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}")
}

fn unconfigured() -> ApiErr {
    ApiErr(StatusCode::SERVICE_UNAVAILABLE, ApiError::new("github_unconfigured", "GitHub sign-in is not configured."))
}

fn upstream() -> ApiErr {
    ApiErr(StatusCode::BAD_GATEWAY, ApiError::new("github_unavailable", "GitHub sign-in failed. Try again."))
}

async fn start(State(state): State<AppState>) -> ApiResult<Response> {
    let client_id = setting(&state, "client_id").filter(|s| !s.is_empty()).ok_or_else(unconfigured)?;
    if state.github_secret.is_none() {
        return Err(unconfigured());
    }
    let authorize = setting(&state, "authorize_url").unwrap_or("https://github.com/login/oauth/authorize");
    let mut url = Url::parse(authorize).map_err(ApiErr::internal)?;
    let token = il_sdk::id("oauth");
    state
        .services
        .store
        .kv_put(&state_key(&token), &json!({"expiresAt": now() + 600}))
        .await
        .map_err(ApiErr::internal)?;
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", &callback_url(&state))
        .append_pair("state", &token);
    let mut response = Redirect::temporary(url.as_str()).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_str(&cookie(&state, STATE_COOKIE, &token, 600, "/api/auth/github"))
            .map_err(ApiErr::internal)?,
    );
    Ok(response)
}

async fn callback(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<Callback>,
) -> ApiResult<Response> {
    if query.error.is_some() {
        return Err(ApiErr::bad_request("GitHub authorization was declined."));
    }
    let code =
        query.code.as_deref().filter(|s| !s.is_empty()).ok_or_else(|| ApiErr::bad_request("Missing GitHub code."))?;
    let token =
        query.state.as_deref().filter(|s| !s.is_empty()).ok_or_else(|| ApiErr::bad_request("Missing OAuth state."))?;
    if cookie_value(&headers, STATE_COOKIE).as_deref() != Some(token) {
        return Err(ApiErr::bad_request("OAuth state mismatch."));
    }
    let key = state_key(token);
    let saved = state.services.store.kv_get(&key).await.map_err(ApiErr::internal)?;
    let expires = saved.as_ref().and_then(|v| v.get("expiresAt")).and_then(Value::as_u64).unwrap_or(0);
    if expires < now() {
        return Err(ApiErr::bad_request("OAuth state expired."));
    }
    state.services.store.kv_put(&key, &Value::Null).await.map_err(ApiErr::internal)?;
    let client_id = setting(&state, "client_id").filter(|s| !s.is_empty()).ok_or_else(unconfigured)?;
    let secret = state.github_secret.as_deref().ok_or_else(unconfigured)?;
    let token_url = setting(&state, "token_url").unwrap_or("https://github.com/login/oauth/access_token");
    let request = state.services.http.post(token_url).map_err(ApiErr::internal)?
        .header(header::ACCEPT, "application/json")
        .json(&json!({"client_id": client_id, "client_secret": secret, "code": code, "redirect_uri": callback_url(&state)}));
    let reply = request.send().await.map_err(|_| upstream())?;
    if !reply.status().is_success() {
        return Err(upstream());
    }
    let access: Value = reply.json().await.map_err(|_| upstream())?;
    let access_token = access.get("access_token").and_then(Value::as_str).ok_or_else(upstream)?;
    let user_url = setting(&state, "user_url").unwrap_or("https://api.github.com/user");
    let request = state
        .services
        .http
        .get(user_url)
        .map_err(ApiErr::internal)?
        .bearer_auth(access_token)
        .header(header::USER_AGENT, "Interlock")
        .header(header::ACCEPT, "application/vnd.github+json");
    let reply = request.send().await.map_err(|_| upstream())?;
    if !reply.status().is_success() {
        return Err(upstream());
    }
    let user: Value = reply.json().await.map_err(|_| upstream())?;
    let id = user.get("id").and_then(Value::as_u64).ok_or_else(upstream)?;
    let login = user.get("login").and_then(Value::as_str).ok_or_else(upstream)?;
    let summary = UserSummary {
        id: format!("github:{id}"),
        name: user.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or(login).into(),
        avatar_url: user.get("avatar_url").and_then(Value::as_str).map(str::to_owned),
    };
    let session = Session { workspace_id: format!("ws_gh_{id}"), user: Some(summary), demo: false };
    let session_token = il_sdk::id("ses");
    state
        .services
        .store
        .kv_put(&session_key(&session_token), &serde_json::to_value(session).map_err(ApiErr::internal)?)
        .await
        .map_err(ApiErr::internal)?;
    let mut response = Redirect::to("/").into_response();
    for value in [
        cookie(&state, SESSION_COOKIE, &session_token, 604800, "/"),
        cookie(&state, STATE_COOKIE, "", 0, "/api/auth/github"),
    ] {
        response.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&value).map_err(ApiErr::internal)?);
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use serde_json::json;
    use tower::ServiceExt;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::{Config, app, build_state};

    #[tokio::test]
    async fn github_sign_in_uses_one_use_state_and_creates_session() {
        let github = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_partial_json(json!({"client_id":"client-test","client_secret":"secret-test","code":"code-test"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"token-test"})))
            .expect(1)
            .mount(&github)
            .await;
        Mock::given(method("GET"))
            .and(path("/user"))
            .and(header("authorization", "Bearer token-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"id":42,"login":"octocat","name":"Octo Cat","avatar_url":"https://example.com/avatar.png"}),
            ))
            .expect(1)
            .mount(&github)
            .await;
        let config = Config::from_value(json!({"auth":{"github":{
            "client_id":"client-test",
            "authorize_url":format!("{}/authorize",github.uri()),
            "token_url":format!("{}/token",github.uri()),
            "user_url":format!("{}/user",github.uri())
        }}}))
        .unwrap();
        let mut state = build_state(config);
        state.services.http = il_sdk::SafeHttp::permissive_for_tests();
        state.github_secret = Some("secret-test".into());
        let router = app(state);
        let start =
            router.clone().oneshot(Request::get("/api/auth/github/start").body(Body::empty()).unwrap()).await.unwrap();
        assert!(start.status().is_redirection());
        let location = Url::parse(start.headers()[header::LOCATION].to_str().unwrap()).unwrap();
        assert_eq!(location.path(), "/authorize");
        assert_eq!(location.query_pairs().find(|(k, _)| k == "client_id").unwrap().1, "client-test");
        let token = location.query_pairs().find(|(k, _)| k == "state").unwrap().1.into_owned();
        let browser_cookie =
            start.headers()[header::SET_COOKIE].to_str().unwrap().split(';').next().unwrap().to_owned();

        let wrong = router
            .clone()
            .oneshot(
                Request::get("/api/auth/github/callback?code=code-test&state=wrong")
                    .header(header::COOKIE, &browser_cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::BAD_REQUEST);
        let callback = format!("/api/auth/github/callback?code=code-test&state={token}");
        let done = router
            .clone()
            .oneshot(Request::get(&callback).header(header::COOKIE, &browser_cookie).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(done.status().is_redirection());
        let session_cookie = done
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .find(|v| v.starts_with("il_session="))
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let me = router
            .clone()
            .oneshot(Request::get("/api/me").header(header::COOKIE, session_cookie).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let body = axum::body::to_bytes(me.into_body(), usize::MAX).await.unwrap();
        let me: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(me["workspaceId"], "ws_gh_42");
        assert_eq!(me["user"]["name"], "Octo Cat");
        let replay = router
            .oneshot(Request::get(&callback).header(header::COOKIE, browser_cookie).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(replay.status(), StatusCode::BAD_REQUEST);
        github.verify().await;
    }
}
