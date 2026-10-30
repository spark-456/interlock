//! Workspaces can't see each other's files or builds (Q10), and a public deploy can turn off the
//! no-sign-in local workspace.

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn app(local_workspace: bool) -> (axum::Router, il_server::AppState, tempfile::TempDir) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = std::fs::read_to_string(root.join("config/interlock.example.toml")).unwrap();
    let mut config = serde_json::to_value(toml::from_str::<toml::Value>(&example).unwrap()).unwrap();
    let data = tempfile::tempdir().unwrap();
    config["server"]["templates_dir"] = root.join("fixtures/templates").to_str().unwrap().into();
    config["server"]["local_workspace"] = local_workspace.into();
    config["storage"]["path"] = data.path().to_str().unwrap().into();
    let state =
        il_server::wiring::build_state_persistent(il_server::Config::from_value(config).unwrap()).await.unwrap();
    (il_server::app(state.clone()), state, data)
}

async fn send(app: &axum::Router, req: Request<Body>) -> (StatusCode, Value, Option<String>) {
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let cookie = resp
        .headers()
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|c| c.split(';').next().unwrap_or_default().to_string());
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    (status, serde_json::from_slice(&bytes).unwrap_or(Value::Null), cookie)
}

fn get(path: &str, cookie: &str) -> Request<Body> {
    Request::get(path).header(header::COOKIE, cookie).body(Body::empty()).unwrap()
}

/// Signs someone in to their own workspace the way sign-in does, and returns their session cookie.
async fn sign_in(state: &il_server::AppState, workspace_id: &str) -> String {
    let token = format!("ses_{workspace_id}");
    let session = json!({"workspaceId": workspace_id, "user": {"id": format!("github:{workspace_id}"), "name": "Pat"}});
    state.services.store.kv_put(&format!("ws:session:{token}"), &session).await.unwrap();
    format!("il_session={token}")
}

#[tokio::test]
async fn q10_people_cannot_read_each_others_files_or_builds() {
    let (app, state, _data) = app(true).await;
    let (a, b) = (sign_in(&state, "ws_gh_1").await, sign_in(&state, "ws_gh_2").await);

    let boundary = "XB";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.txt\"\r\nContent-Type: text/plain\r\n\r\nsecret\r\n--{boundary}--\r\n"
    );
    let upload = Request::post("/api/files")
        .header(header::COOKIE, &a)
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap();
    let (status, refs, _) = send(&app, upload).await;
    assert_eq!(status, StatusCode::CREATED);
    let file = format!("/api/files/{}", refs[0]["id"].as_str().unwrap());
    assert_eq!(send(&app, get(&file, &a)).await.0, StatusCode::OK);
    assert_eq!(send(&app, get(&file, &b)).await.0, StatusCode::NOT_FOUND);

    let build = Request::post("/api/builds")
        .header(header::COOKIE, &a)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"goal": "Answer support questions from our help center"}).to_string()))
        .unwrap();
    let (status, created, _) = send(&app, build).await;
    assert_eq!(status, StatusCode::ACCEPTED, "{created}");
    let events = format!("/api/builds/{}/events", created["buildId"].as_str().unwrap());
    assert_eq!(send(&app, get(&events, &b)).await.0, StatusCode::NOT_FOUND);
    let answers = Request::post(format!("/api/builds/{}/answers", created["buildId"].as_str().unwrap()))
        .header(header::COOKIE, &b)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json!({"answers": {}}).to_string()))
        .unwrap();
    assert_eq!(send(&app, answers).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn q10_a_public_deploy_has_no_sign_in_free_workspace() {
    let (app, state, _data) = app(false).await;
    let (status, error, _) = send(&app, Request::get("/api/flows").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(error["error"]["code"], "sign_in_required");
    let session = sign_in(&state, "ws_gh_1").await;
    assert_eq!(send(&app, get("/api/flows", &session)).await.0, StatusCode::OK, "signed-in people still work");
    let oauth = send(&app, Request::get("/api/oauth/google/start").body(Body::empty()).unwrap()).await.0;
    assert_eq!(oauth, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn q18_an_expired_session_can_be_cleared() {
    let stale = "il_session=gone";
    let (app, _state, _data) = app(true).await;
    let (status, error, _) = send(&app, get("/api/me", stale)).await;
    assert_eq!((status, error["error"]["code"].as_str()), (StatusCode::UNAUTHORIZED, Some("session_expired")));
    let leave = Request::post("/api/auth/sign-out").header(header::COOKIE, stale).body(Body::empty()).unwrap();
    let (status, _, cleared) = send(&app, leave).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(cleared.as_deref(), Some("il_session="), "the browser drops the stale cookie");
}
