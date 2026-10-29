//! In-memory integration tests for the internal API (PRD §8.5).

use axum::Router;
use axum::body::Body;
use axum::extract::connect_info::ConnectInfo;
use axum::http::{Request, StatusCode, header};
use il_server::{Config, app, build_state};
use serde_json::{Value, json};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
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
async fn q04_reset_demo_session_cannot_access_local_workspace() {
    let state = build_state(Config::from_value(json!({})).unwrap());
    let app = app(state.clone());
    let (_, me, cookie) = call(&app, "POST", "/api/demo/session", None, None).await;
    let id = me["workspaceId"].as_str().unwrap();
    let pair = cookie.unwrap().split(';').next().unwrap().to_string();
    state.services.files.delete_workspace_files(id).await.unwrap();
    state.services.store.delete_demo_workspace(id).await.unwrap();
    let (status, _, _) = call(&app, "GET", "/api/me", None, Some(&pair)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
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
    let (s, created, _) =
        call(&app, "POST", "/api/builds", Some(json!({ "goal": "Reply to support emails" })), None).await;
    assert_eq!(s, StatusCode::ACCEPTED);
    assert!(created["buildId"].as_str().unwrap().starts_with("bld_"));

    let (s, err, _) = call(&app, "POST", "/api/templates/tpl_x/use", None, None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
    assert_eq!(err["error"]["code"], "not_found");
}

#[tokio::test]
async fn r06_fake_build_selects_a_template_and_emits_ready() {
    let dir = tempfile::tempdir().unwrap();
    let templates = dir.path().join("templates");
    std::fs::create_dir(&templates).unwrap();
    for (id, name, goal) in [
        ("support", "Support chat", "Answer support chat questions from documents"),
        ("promos", "Promo clips", "Make a product promo video"),
    ] {
        let template = json!({
            "id": format!("flw_{id}"), "workspaceId": "template", "version": 1,
            "name": name, "kind": "automation", "edges": [],
            "nodes": [{"id": "n_start", "type": "trigger.manual", "name": "Start", "settings": {}}],
            "meta": {"createdBy": "template", "goal": goal}
        });
        std::fs::write(templates.join(format!("{id}.json")), template.to_string()).unwrap();
    }
    let state =
        build_state(Config::from_value(json!({"server": {"templates_dir": templates.to_str().unwrap()}})).unwrap());
    let app = app(state.clone());
    let (status, body, _) =
        call(&app, "POST", "/api/builds", Some(json!({"goal": "Build a support chat agent"})), None).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let build_id = body["buildId"].as_str().unwrap();
    for _ in 0..100 {
        let events = state.services.store.build_events(build_id, 0).await.unwrap();
        if let Some(event) = events.last() {
            match &event.kind {
                il_core::BuildEventKind::Ready { flow_id } => {
                    let flow = state.services.store.get_flow(flow_id).await.unwrap().unwrap();
                    assert_eq!(flow.name, "Support chat");
                    assert_eq!(flow.meta.goal.as_deref(), Some("Build a support chat agent"));
                    assert!(events.iter().any(|e| matches!(e.kind, il_core::BuildEventKind::FlowPatch { .. })));
                    return;
                }
                il_core::BuildEventKind::Failed { error } => panic!("fake build failed: {error}"),
                _ => {}
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    panic!("fake build never completed");
}

#[tokio::test]
async fn templates_are_listed_and_copied_into_the_workspace() {
    let root = tempfile::tempdir().unwrap();
    let (templates, recorded) = (root.path().join("templates"), root.path().join("recorded"));
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::create_dir_all(&recorded).unwrap();
    let mut flow = manual_flow();
    flow["id"] = json!("flw_hello");
    flow["workspaceId"] = json!("template");
    flow["version"] = json!(1);
    flow["kind"] = json!("automation");
    flow["meta"] = json!({ "goal": "Say hello every morning." });
    std::fs::write(templates.join("tpl_hello.json"), flow.to_string()).unwrap();
    std::fs::write(templates.join("notes.txt"), "not a template").unwrap();
    std::fs::write(recorded.join("tpl_hello.json"), "[]").unwrap();
    let config = Config::from_value(json!({"server":{"templates_dir":templates.to_str().unwrap()}})).unwrap();
    let app = app(build_state(config));

    let (s, list, _) = call(&app, "GET", "/api/templates", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], "tpl_hello");
    assert_eq!(list[0]["description"], "Say hello every morning.");
    assert_eq!(list[0]["hasRecordedRun"], true);

    let (s, copy, _) = call(&app, "POST", "/api/templates/tpl_hello/use", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert!(copy["id"].as_str().unwrap().starts_with("flw_"));
    assert_ne!(copy["id"], flow["id"]);
    let (_, flows, _) = call(&app, "GET", "/api/flows", None, None).await;
    assert!(flows.as_array().unwrap().iter().any(|f| f["id"] == copy["id"]));
    for missing in ["tpl_nope", "..%2Ftpl_hello", "notes"] {
        let (s, _, _) = call(&app, "POST", &format!("/api/templates/{missing}/use"), None, None).await;
        assert_eq!(s, StatusCode::NOT_FOUND, "{missing}");
    }
}

#[tokio::test]
async fn r05_support_template_seeds_openapi_spec_for_agent_tools() {
    let templates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/templates");
    let state =
        build_state(Config::from_value(json!({"server":{"templates_dir":templates.to_str().unwrap()}})).unwrap());
    let app = app(state.clone());
    let (status, body, _) = call(&app, "POST", "/api/templates/tpl_support_agent/use", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let flow: il_core::Flow = serde_json::from_value(body).unwrap();
    let spec = flow.node("n_billing").unwrap().settings["apiTools"]["spec"].as_str().unwrap();
    assert!(spec.starts_with("file_"));
    let tools = state.services.flow_tools.for_flow(&flow.workspace_id, &flow).await.unwrap();
    assert_eq!(tools.iter().map(|tool| tool.name()).collect::<Vec<_>>(), ["lookup_customer", "list_invoices"]);
}

#[tokio::test]
async fn public_endpoints_are_limited_per_ip() {
    let config =
        Config::from_value(json!({"server":{"public_requests_per_minute":2,"demo_sessions_per_hour":1}})).unwrap();
    let app = app(build_state(config));
    for (ip, expected) in [
        (Ipv4Addr::new(10, 0, 0, 1), StatusCode::NOT_FOUND),
        (Ipv4Addr::new(10, 0, 0, 1), StatusCode::NOT_FOUND),
        (Ipv4Addr::new(10, 0, 0, 1), StatusCode::TOO_MANY_REQUESTS),
        (Ipv4Addr::new(10, 0, 0, 2), StatusCode::NOT_FOUND),
    ] {
        let mut req = Request::get("/v1/unknown").body(Body::empty()).unwrap();
        req.extensions_mut().insert(ConnectInfo(SocketAddr::new(IpAddr::V4(ip), 1234)));
        assert_eq!(app.clone().oneshot(req).await.unwrap().status(), expected);
    }
    let (first, _, _) = call(&app, "POST", "/api/demo/session", None, None).await;
    let (second, error, _) = call(&app, "POST", "/api/demo/session", None, None).await;
    assert_eq!(first, StatusCode::OK);
    assert_eq!(second, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(error["error"]["code"], "rate_limited");
}

#[tokio::test]
async fn demo_session_seeds_template_flows_when_fixtures_exist() {
    let dir = tempfile::tempdir().unwrap();
    let template = json!({
        "id":"template_1", "workspaceId":"template", "version":1,
        "name":"Demo flow", "kind":"automation",
        "nodes":[{"id":"n_start","type":"trigger.manual","name":"Start","settings":{}}], "edges":[]
    });
    let (templates, recorded) = (dir.path().join("templates"), dir.path().join("recorded"));
    std::fs::create_dir_all(&templates).unwrap();
    std::fs::create_dir_all(&recorded).unwrap();
    std::fs::write(templates.join("demo.json"), template.to_string()).unwrap();
    let ev = |seq: u64, node: Option<&str>, kind: &str, data: Value| json!({"seq": seq, "ts": format!("2026-10-26T09:00:0{seq}Z"), "runId": "rec", "nodeId": node, "type": kind, "data": data});
    let recording = json!([
        ev(1, None, "run.started", json!({"flowId": "template_1", "trigger": "trigger.manual"})),
        ev(2, Some("n_start"), "node.started", json!({})),
        ev(3, Some("n_start"), "node.succeeded", json!({"count": 1, "ms": 5, "costUsd": 0.0})),
        ev(4, None, "run.succeeded", json!({"ms": 3000, "costUsd": 0.002})),
    ]);
    std::fs::write(recorded.join("demo.json"), recording.to_string()).unwrap();
    let config = Config::from_value(json!({"server":{"templates_dir":templates.to_str().unwrap()}})).unwrap();
    let app = app(build_state(config));
    let (_, first, cookie) = call(&app, "POST", "/api/demo/session", None, None).await;
    let cookie = cookie.unwrap();
    let cookie = cookie.split(';').next().unwrap();
    let (_, flows, _) = call(&app, "GET", "/api/flows", None, Some(cookie)).await;
    assert_eq!(flows.as_array().unwrap().len(), 1);
    assert_eq!(flows[0]["name"], "Demo flow");
    // The recorded run comes with it, as a finished replay run of the seeded flow.
    let path = format!("/api/flows/{}/runs", flows[0]["id"].as_str().unwrap());
    let (_, runs, _) = call(&app, "GET", &path, None, Some(cookie)).await;
    assert_eq!(runs.as_array().unwrap().len(), 1);
    assert_eq!((runs[0]["status"].as_str(), runs[0]["replay"].as_bool()), (Some("succeeded"), Some(true)));
    assert_eq!(runs[0]["costUsd"], 0.002);
    let (_, second, _) = call(&app, "POST", "/api/demo/session", None, None).await;
    assert_ne!(first["workspaceId"], second["workspaceId"]);
}

#[tokio::test]
async fn startup_restores_active_triggers_and_running_runs() {
    use il_core::{Run, RunStatus};

    let dir = tempfile::tempdir().unwrap();
    let config = Config::from_value(json!({"storage":{"path":dir.path().to_str().unwrap()}})).unwrap();
    let state = il_server::wiring::build_state_persistent(config.clone()).await.unwrap();
    let app = app(state.clone());
    let body = json!({
        "name":"Recovery", "nodes":[{"id":"n_hook","type":"trigger.webhook","name":"Hook","settings":{"method":"POST"}}], "edges":[]
    });
    let (_, created, _) = call(&app, "POST", "/api/flows", Some(body), None).await;
    let flow_id = created["id"].as_str().unwrap().to_owned();
    call(&app, "POST", &format!("/api/flows/{flow_id}/activate"), None, None).await;
    let run = Run {
        id: "run_recovery".into(),
        flow_id: flow_id.clone(),
        workspace_id: "ws_local".into(),
        flow_version: 1,
        status: RunStatus::Running,
        trigger: "test".into(),
        started_at: state.services.clock.now(),
        finished_at: None,
        cost_usd: 0.0,
        error: None,
        replay: false,
        state: json!({"trigger_node":"n_hook","outputs":{"n_hook":{"out":[]}},"skipped":[],"partial":{},"response":[]}),
    };
    state.services.store.put_run(&run).await.unwrap();
    drop(app);
    drop(state);
    let state = il_server::wiring::build_state_persistent(config).await.unwrap();
    assert!(!state.triggers.is_running(&flow_id).await);
    il_server::wiring::recover_startup(&state).await.unwrap();
    assert!(state.triggers.is_running(&flow_id).await);
    for _ in 0..100 {
        if state.services.store.get_run(&run.id).await.unwrap().unwrap().status == RunStatus::Succeeded {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("running run was not recovered");
}

#[tokio::test]
async fn persistent_server_keeps_sessions_across_restarts() {
    let dir = tempfile::tempdir().unwrap();
    let config = Config::from_value(json!({"storage":{"path":dir.path().to_str().unwrap()}})).unwrap();
    let state = il_server::wiring::build_state_persistent(config.clone()).await.unwrap();
    let (_, me, cookie) = call(&app(state), "POST", "/api/demo/session", None, None).await;
    let state = il_server::wiring::build_state_persistent(config).await.unwrap();
    let pair = cookie.unwrap().split(';').next().unwrap().to_owned();
    let (_, again, _) = call(&app(state), "GET", "/api/me", None, Some(&pair)).await;
    assert_eq!(again["workspaceId"], me["workspaceId"]);
}

#[tokio::test]
async fn key_connections_are_saved_listed_and_never_returned() {
    let app = app(build_state(Config::from_value(json!({})).unwrap()));
    let (s, list, _) = call(&app, "GET", "/api/connections", None, None).await;
    assert_eq!((s, list), (StatusCode::OK, json!([])));
    let body = json!({"app": "telegram", "label": "Support bot", "secret": {"token": "123:ABC-secret"}});
    let (s, created, _) = call(&app, "POST", "/api/connections", Some(body), None).await;
    assert_eq!(s, StatusCode::CREATED);
    assert_eq!(
        (created["app"].as_str(), created["label"].as_str(), created["status"].as_str()),
        (Some("telegram"), Some("Support bot"), Some("connected"))
    );
    let (_, list, _) = call(&app, "GET", "/api/connections", None, None).await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert!(!list.to_string().contains("ABC-secret"), "secrets never leave the server");
    let (s, _, _) =
        call(&app, "POST", "/api/connections", Some(json!({"app": "telegram", "secret": {"token": " "}})), None).await;
    assert_eq!(s, StatusCode::BAD_REQUEST);

    let (_, _, cookie) = call(&app, "POST", "/api/demo/session", None, None).await;
    let cookie = cookie.unwrap();
    let demo = cookie.split(';').next().unwrap();
    let body = json!({"app": "discord", "secret": {"url": "https://discord.example/hook"}});
    let (s, err, _) = call(&app, "POST", "/api/connections", Some(body), Some(demo)).await;
    assert_eq!((s, err["error"]["code"].as_str()), (StatusCode::FORBIDDEN, Some("demo_restricted")));
}

#[tokio::test]
async fn usage_reports_thirty_days_by_day_and_flow_with_the_caps() {
    let state = build_state(Config::from_value(json!({"caps": {"run_usd": 0.5}})).unwrap());
    let app = app(state.clone());
    let (_, me, _) = call(&app, "GET", "/api/me", None, None).await;
    let ws = me["workspaceId"].as_str().unwrap().to_string();
    let (_, flow, _) = call(&app, "POST", "/api/flows", Some(manual_flow()), None).await;
    let flow_id = flow["id"].as_str().unwrap().to_string();
    let entry = |flow: Option<&str>, cost: f64, days_ago: i64| il_sdk::services::store::SpendEntry {
        workspace_id: ws.clone(),
        flow_id: flow.map(String::from),
        run_id: None,
        build_id: None,
        role: "node".into(),
        cost_usd: cost,
        at: chrono::Utc::now() - chrono::Duration::days(days_ago),
    };
    for e in [
        entry(Some(&flow_id), 0.25, 0),
        entry(Some(&flow_id), 0.5, 2),
        entry(None, 0.125, 0),
        entry(Some(&flow_id), 9.0, 40),
    ] {
        state.services.store.record_spend(&e).await.unwrap();
    }
    let (s, usage, _) = call(&app, "GET", "/api/usage", None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(usage["totalUsd"], 0.875, "the 40-day-old entry is outside the window");
    let days = usage["days"].as_array().unwrap();
    assert_eq!(days.len(), 30);
    assert_eq!(days[29]["costUsd"], 0.375);
    assert_eq!(days[27]["costUsd"], 0.5);
    assert_eq!(usage["flows"], json!([{ "flowId": flow_id, "name": "Hello", "costUsd": 0.75 }]));
    assert_eq!(usage["caps"]["runUsd"], 0.5);
}

#[tokio::test]
async fn webhook_steps_get_a_stable_address_before_activation() {
    let app = app(build_state(Config::from_value(json!({})).unwrap()));
    let flow = json!({"name": "Hooked", "nodes": [
        {"id": "n_hook", "type": "trigger.webhook", "name": "Incoming", "settings": {}},
        {"id": "n_text", "type": "data.template", "name": "Echo", "settings": {"template": "hi"}}],
        "edges": [{"id": "e1", "from": {"node": "n_hook", "port": "out"}, "to": {"node": "n_text", "port": "in"}}]});
    let (_, created, _) = call(&app, "POST", "/api/flows", Some(flow), None).await;
    let path = format!("/api/flows/{}/hooks", created["id"].as_str().unwrap());
    let (s, hooks, _) = call(&app, "GET", &path, None, None).await;
    assert_eq!(s, StatusCode::OK);
    assert_eq!(hooks[0]["nodeId"], "n_hook");
    let url = hooks[0]["path"].as_str().unwrap().to_string();
    assert!(url.starts_with(&format!("/hooks/{}/whk", created["id"].as_str().unwrap())));
    let (_, again, _) = call(&app, "GET", &path, None, None).await;
    assert_eq!(again[0]["path"], url.as_str(), "the address is stable");
    let (s, _, _) = call(&app, "POST", &url, Some(json!({"x": 1})), None).await;
    assert_eq!(s, StatusCode::NOT_FOUND, "refused while the flow is off");
    let (s, _, _) = call(&app, "GET", "/api/flows/flw_nope/hooks", None, None).await;
    assert_eq!(s, StatusCode::NOT_FOUND);
}
