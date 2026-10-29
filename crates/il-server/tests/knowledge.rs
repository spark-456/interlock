//! The persistent server uses the real knowledge service (K03): the support template's help center is
//! ingested offline from the recorded `.example` site, its agent answers with sources, the base is
//! searchable, and later runs reuse it.

use serde_json::Value;
use tower::ServiceExt;

async fn call(app: &axum::Router, method: &str, path: &str) -> Value {
    let req = axum::http::Request::builder().method(method).uri(path).body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success(), "{method} {path}: {}", resp.status());
    serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap()
}

async fn send(app: &axum::Router, path: &str, body: Value) -> (u16, Value) {
    let req = axum::http::Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    (
        status,
        serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap_or_default(),
    )
}

async fn persistent_app() -> (axum::Router, tempfile::TempDir) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = std::fs::read_to_string(root.join("config/interlock.example.toml")).unwrap();
    let mut config = serde_json::to_value(toml::from_str::<toml::Value>(&example).unwrap()).unwrap();
    let data = tempfile::tempdir().unwrap();
    config["server"]["templates_dir"] = root.join("fixtures/templates").to_str().unwrap().into();
    config["storage"]["path"] = data.path().to_str().unwrap().into();
    let state =
        il_server::wiring::build_state_persistent(il_server::Config::from_value(config).unwrap()).await.unwrap();
    (il_server::app(state), data)
}

#[tokio::test]
async fn k04_knowledge_api_creates_lists_and_adds_sources() {
    let (app, _data) = persistent_app().await;
    let text = serde_json::json!({"kind": "text", "title": "Refunds", "text": "Refunds within 30 days."});
    let (status, made) = send(&app, "/api/knowledge", serde_json::json!({"name": "Help", "sources": [text]})).await;
    assert_eq!(status, 201, "{made}");
    let kb = made["knowledgeBase"].as_str().unwrap();
    let more = serde_json::json!({"knowledgeBase": kb, "sources": [{"kind": "text", "title": "Plans", "text": "Pro is $20."}]});
    assert_eq!(send(&app, "/api/knowledge", more).await.0, 201);
    let list = call(&app, "GET", "/api/knowledge").await;
    assert_eq!(list[0]["id"], kb);
    assert_eq!(list[0]["sources"], 2);
    let other = serde_json::json!({"knowledgeBase": "kb_not_mine", "sources": []});
    assert_eq!(send(&app, "/api/knowledge", other).await.0, 404);
    assert_eq!(send(&app, "/api/knowledge", serde_json::json!({})).await.0, 400);
}

async fn test_run(app: &axum::Router, flow_id: &str) -> Value {
    let run = call(app, "POST", &format!("/api/flows/{flow_id}/test")).await;
    let path = format!("/api/runs/{}", run["runId"].as_str().unwrap());
    for _ in 0..2400 {
        let detail = call(app, "GET", &path).await;
        if !["queued", "running"].contains(&detail["run"]["status"].as_str().unwrap()) {
            return detail;
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
    panic!("the run did not finish");
}

fn step<'a>(detail: &'a Value, node: &str) -> Option<&'a Value> {
    detail["nodeRuns"].as_array().unwrap().iter().find(|n| n["nodeId"] == node)
}

#[tokio::test]
async fn k03_support_template_answers_from_its_knowledge_base() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = std::fs::read_to_string(root.join("config/interlock.example.toml")).unwrap();
    let mut config = serde_json::to_value(toml::from_str::<toml::Value>(&example).unwrap()).unwrap();
    let data = tempfile::tempdir().unwrap();
    config["server"]["templates_dir"] = root.join("fixtures/templates").to_str().unwrap().into();
    config["storage"]["path"] = data.path().to_str().unwrap().into();
    let state =
        il_server::wiring::build_state_persistent(il_server::Config::from_value(config).unwrap()).await.unwrap();
    let knowledge = state.services.knowledge.clone();
    let app = il_server::app(state);

    let flow = call(&app, "POST", "/api/templates/tpl_support_agent/use").await;
    let flow_id = flow["id"].as_str().unwrap();
    let detail = test_run(&app, flow_id).await;
    assert_eq!(detail["run"]["status"], "succeeded", "{}", detail["run"]["error"]);

    let kb = step(&detail, "n_kb").unwrap()["output"]["kb"][0]["json"]["knowledgeBase"].as_str().unwrap().to_string();
    assert!(!knowledge.search(&kb, "refund", 5).await.unwrap().is_empty(), "the help center is searchable");
    let agent = ["n_support", "n_billing"].iter().find_map(|n| step(&detail, n)).expect("an agent answered");
    let answer = &agent["output"]["out"][0]["json"];
    assert!(!answer["sources"].as_array().unwrap().is_empty(), "the answer cites the help center: {answer}");

    let again = test_run(&app, flow_id).await;
    assert_eq!(step(&again, "n_kb").unwrap()["output"]["kb"][0]["json"]["knowledgeBase"], kb.as_str(), "reused");
}
