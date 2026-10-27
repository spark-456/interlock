//! In-memory integration tests for the internal API (PRD §8.5).

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use il_server::{Config, app, build_state};
use serde_json::{Value, json};
use tower::ServiceExt;

fn server() -> Router {
    app(build_state(Config::from_value(json!({})).unwrap()))
}

async fn call(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> (StatusCode, Value, Option<String>) {
    let mut req = Request::builder().method(method).uri(path);
    if let Some(c) = cookie {
        req = req.header(header::COOKIE, c);
    }
    let req = match body {
        Some(b) => req.header(header::CONTENT_TYPE, "application/json").body(Body::from(b.to_string())).unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status();
    let set_cookie = resp.headers().get(header::SET_COOKIE).and_then(|v| v.to_str().ok()).map(str::to_string);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into()));
    (status, body, set_cookie)
}

fn manual_flow() -> Value {
    json!({
        "name": "Hello",
        "nodes": [{ "id": "n_start", "type": "trigger.manual", "name": "Run by hand", "settings": {} }],
        "edges": []
    })
}

#[tokio::test]
async fn flow_crud_and_versions() {
    let app = server();
    let (s, created, _) = call(&app, "POST", "/api/flows", Some(manual_flow()), None).await;
    assert_eq!(s, StatusCode::CREATED);
    let id = created["id"].as_str().unwrap().to_string();
    assert!(id.starts_with("flw_"));
    assert_eq!(created["version"], 1);
    assert_eq!(created["workspaceId"], "ws_local");

    let (s, list, _) = call(&app, "GET", "/api/flows", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["active"], false);

    let mut edited = created.clone();
    edited["name"] = json!("Hello again");
    let (s, updated, _) = call(&app, "PUT", &format!("/api/flows/{id}"), Some(edited), None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(updated["version"], 2);
    let (_, versions, _) = call(&app, "GET", &format!("/api/flows/{id}/versions"), None, None).await;
    assert_eq!(versions.as_array().unwrap().len(), 2);

    let (s, _, _) = call(&app, "DELETE", &format!("/api/flows/{id}"), None, None).await;
    assert_eq!(s, StatusCode::NO_CONTENT);
    let (s, err, _) = call(&app, "GET", &format!("/api/flows/{id}"), None, None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert_eq!(err["error"]["code"], "not_found");
}

#[tokio::test]
async fn test_run_completes_and_streams_events() {
    let app = server();
    let (_, created, _) = call(&app, "POST", "/api/flows", Some(manual_flow()), None).await;
    let id = created["id"].as_str().unwrap();
    let (s, run, _) =
        call(&app, "POST", &format!("/api/flows/{id}/test"), Some(json!({ "items": [{ "n": 1 }] })), None).await;
    assert_eq!(s, StatusCode::ACCEPTED);
    let run_id = run["runId"].as_str().unwrap().to_string();

    let mut status = Value::Null;
    for _ in 0..100 {
        let (_, detail, _) = call(&app, "GET", &format!("/api/runs/{run_id}"), None, None).await;
        status = detail["run"]["status"].clone();
        if status == "succeeded" {
            assert!(detail["run"]["state"].is_null(), "engine checkpoint is not exposed");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert_eq!(status, "succeeded");

    // The SSE stream replays history and closes after the terminal event.
    let (s, body, _) = call(&app, "GET", &format!("/api/runs/{run_id}/events?after=0"), None, None).await;
    assert_eq!(s, StatusCode::OK);
    let text = body.as_str().unwrap();
    assert!(text.contains("\"type\":\"run.started\""));
    assert!(text.contains("\"type\":\"run.succeeded\""));
    assert!(text.contains("id: 1"));
    // Resume after a cursor skips earlier events.
    let (_, tail, _) = call(&app, "GET", &format!("/api/runs/{run_id}/events?after=2"), None, None).await;
    assert!(!tail.as_str().unwrap().contains("run.started"));

    let (_, runs, _) = call(&app, "GET", &format!("/api/flows/{id}/runs"), None, None).await;
    assert_eq!(runs[0]["status"], "succeeded");
}

#[tokio::test]
async fn demo_sessions_are_isolated_workspaces() {
    let app = server();
    let (s, me, cookie) = call(&app, "POST", "/api/demo/session", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(me["demo"], true);
    let cookie = cookie.unwrap();
    assert!(cookie.contains("HttpOnly"));
    let pair = cookie.split(';').next().unwrap().to_string();
    let (_, me2, _) = call(&app, "GET", "/api/me", None, Some(&pair)).await;
    assert_eq!(me2["workspaceId"], me["workspaceId"]);

    let (_, created, _) = call(&app, "POST", "/api/flows", Some(manual_flow()), Some(&pair)).await;
    let id = created["id"].as_str().unwrap();
    let (s, _, _) = call(&app, "GET", &format!("/api/flows/{id}"), None, Some(&pair)).await;
    assert_eq!(s, StatusCode::OK);
    let (s, _, _) = call(&app, "GET", &format!("/api/flows/{id}"), None, None).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "other workspaces cannot see it");
}

#[tokio::test]
async fn activate_and_deactivate() {
    let app = server();
    let body = json!({
        "name": "Hook",
        "nodes": [{ "id": "n_hook", "type": "trigger.webhook", "name": "Webhook", "settings": { "method": "POST" } }],
        "edges": []
    });
    let (_, created, _) = call(&app, "POST", "/api/flows", Some(body), None).await;
    let id = created["id"].as_str().unwrap();
    let (s, out, _) = call(&app, "POST", &format!("/api/flows/{id}/activate"), None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(out, json!({ "active": true, "triggers": 1 }));
    let (_, list, _) = call(&app, "GET", "/api/flows", None, None).await;
    assert_eq!(list[0]["active"], true);
    let (_, out, _) = call(&app, "POST", &format!("/api/flows/{id}/deactivate"), None, None).await;
    assert_eq!(out["active"], false);
}

#[tokio::test]
async fn files_upload_and_download() {
    let app = server();
    let boundary = "XBOUNDARY";
    let body = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"notes.txt\"\r\nContent-Type: text/plain\r\n\r\nhello larkspur\r\n--{boundary}--\r\n"
    );
    let req = Request::post("/api/files")
        .header(header::CONTENT_TYPE, format!("multipart/form-data; boundary={boundary}"))
        .body(Body::from(body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
    let refs: Value =
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap();
    assert_eq!(refs[0]["name"], "notes.txt");
    let id = refs[0]["id"].as_str().unwrap();
    let resp =
        app.clone().oneshot(Request::get(format!("/api/files/{id}")).body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(resp.headers()[header::CONTENT_TYPE], "text/plain");
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"hello larkspur");
}

#[tokio::test]
async fn approvals_builds_and_stubs_use_the_error_shape() {
    let app = server();
    let (s, list, _) = call(&app, "GET", "/api/approvals", None, None).await;
    assert_eq!((s, list), (StatusCode::OK, json!([])));
    let (s, err, _) =
        call(&app, "POST", "/api/approvals/apr_x/decide", Some(json!({ "decision": "approve" })), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert_eq!(err["error"]["code"], "not_found");

    let (s, err, _) = call(&app, "POST", "/api/builds", Some(json!({ "goal": " " })), None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);
    assert_eq!(err["error"]["code"], "bad_request");
    let (s, err, _) = call(&app, "POST", "/api/builds", Some(json!({ "goal": "Reply to support emails" })), None).await;
    assert_eq!(s, StatusCode::BAD_GATEWAY, "the architect is not wired yet");
    assert_eq!(err["error"]["code"], "architect_failed");

    let (s, err, _) = call(&app, "POST", "/api/templates/tpl_x/use", None, None).await;
    assert_eq!(s, StatusCode::NOT_IMPLEMENTED);
    assert_eq!(err["error"]["code"], "not_implemented");
}
