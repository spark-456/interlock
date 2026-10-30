//! Sign-in end to end against mock providers.

use axum::body::Body;
// `header` is both axum's header-name module and wiremock's header matcher (separate namespaces).
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;
use url::Url;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

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
    let browser_cookie = start.headers()[header::SET_COOKIE].to_str().unwrap().split(';').next().unwrap().to_owned();

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
    assert!(wrong.headers()[header::LOCATION].to_str().unwrap().starts_with("/signin?error="));
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
    assert!(replay.headers()[header::LOCATION].to_str().unwrap().starts_with("/signin?error="), "a state works once");
    github.verify().await;
}

/// Signs in through `/api/auth/<provider>` with mock endpoints and returns `/api/me`.
async fn sign_in_through(router: &axum::Router, provider: &str) -> Value {
    let start = router
        .clone()
        .oneshot(Request::get(format!("/api/auth/{provider}/start")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(start.status().is_redirection(), "{}", start.status());
    let location = Url::parse(start.headers()[header::LOCATION].to_str().unwrap()).unwrap();
    let token = location.query_pairs().find(|(k, _)| k == "state").unwrap().1.into_owned();
    let browser = start.headers()[header::SET_COOKIE].to_str().unwrap().split(';').next().unwrap().to_owned();
    let done = router
        .clone()
        .oneshot(
            Request::get(format!("/api/auth/{provider}/callback?code=code-test&state={token}"))
                .header(header::COOKIE, &browser)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(done.headers()[header::LOCATION], "/flows");
    let session = done
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
        .oneshot(Request::get("/api/me").header(header::COOKIE, session).body(Body::empty()).unwrap())
        .await
        .unwrap();
    serde_json::from_slice(&axum::body::to_bytes(me.into_body(), usize::MAX).await.unwrap()).unwrap()
}

#[tokio::test]
async fn q34_google_sign_in_asks_only_who_you_are_and_makes_your_workspace() {
    let google = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(wiremock::matchers::body_string_contains("grant_type=authorization_code"))
        .and(wiremock::matchers::body_string_contains("client_secret=secret-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"access_token":"token-test"})))
        .expect(1)
        .mount(&google)
        .await;
    Mock::given(method("GET"))
        .and(path("/userinfo"))
        .and(header("authorization", "Bearer token-test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            json!({"sub":"1077","name":"Pat Lee","email":"pat@example.com","picture":"https://example.com/p.png"}),
        ))
        .expect(1)
        .mount(&google)
        .await;
    let config = Config::from_value(json!({"auth":{"google":{
        "client_id":"client-test",
        "authorize_url":format!("{}/authorize",google.uri()),
        "token_url":format!("{}/token",google.uri()),
        "user_url":format!("{}/userinfo",google.uri())
    }}}))
    .unwrap();
    let mut state = build_state(config);
    state.services.http = il_sdk::SafeHttp::permissive_for_tests();
    state.google_secret = Some("secret-test".into());
    let router = app(state);
    let start =
        router.clone().oneshot(Request::get("/api/auth/google/start").body(Body::empty()).unwrap()).await.unwrap();
    let location = Url::parse(start.headers()[header::LOCATION].to_str().unwrap()).unwrap();
    let scope = location.query_pairs().find(|(k, _)| k == "scope").unwrap().1.into_owned();
    assert_eq!(scope, "openid email profile", "sign-in never asks for Gmail or Drive");
    let me = sign_in_through(&router, "google").await;
    assert_eq!((me["workspaceId"].as_str(), me["user"]["name"].as_str()), (Some("ws_gg_1077"), Some("Pat Lee")));
    assert_eq!(me["user"]["email"], "pat@example.com", "the profile shows the email Google shared");
    google.verify().await;
}

#[tokio::test]
async fn q34_only_configured_providers_are_offered() {
    let config =
        Config::from_value(json!({"auth":{"github":{"client_id":"client-test"},"google":{"client_id":"g"}}})).unwrap();
    let mut state = build_state(config);
    state.github_secret = Some("secret-test".into());
    state.google_secret = None;
    let router = app(state);
    let reply = router.clone().oneshot(Request::get("/api/auth/providers").body(Body::empty()).unwrap()).await.unwrap();
    let list: Value =
        serde_json::from_slice(&axum::body::to_bytes(reply.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(list, json!([{"id":"github","label":"GitHub"}]));
    let google =
        router.clone().oneshot(Request::get("/api/auth/google/start").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(google.status(), StatusCode::SERVICE_UNAVAILABLE);
    let unknown = router.oneshot(Request::get("/api/auth/x/start").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
}
