//! Every checked-in template (`fixtures/templates/*.json`) is a valid flow for the registered catalog,
//! and Test run on a fresh copy runs it end to end with its trigger's sample items (Q02).

use il_architect::catalog::Catalog;
use il_architect::validate::validate;
use il_core::Flow;

#[test]
fn every_template_validates_against_the_catalog() {
    il_server::wiring::link_all();
    let catalog = Catalog::registered();
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/templates");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let flow: Flow = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(validate(&flow, &catalog), vec![], "{}", path.display());
        assert!(flow.meta.goal.as_deref().is_some_and(|g| !g.is_empty()), "{} needs a goal", path.display());
        count += 1;
    }
    assert_eq!(count, 10, "the gallery has ten templates (MK-2)");
}

#[test]
fn recorded_runs_belong_to_their_templates() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut count = 0;
    for entry in std::fs::read_dir(root.join("recorded")).unwrap() {
        let path = entry.unwrap().path();
        let events: Vec<il_core::events::RunEvent> = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let template: Flow =
            serde_json::from_slice(&std::fs::read(root.join("templates").join(path.file_name().unwrap())).unwrap())
                .unwrap();
        for e in &events {
            if let Some(node) = &e.node_id {
                assert!(template.node(node).is_some(), "{}: no step {node}", path.display());
            }
        }
        assert!(events.windows(2).all(|w| w[0].seq < w[1].seq && w[0].ts <= w[1].ts), "{}", path.display());
        let last = &events.last().unwrap().kind;
        assert!(matches!(last, il_core::events::RunEventKind::RunSucceeded { .. }), "{}", path.display());
        count += 1;
    }
    assert_eq!(count, 2, "F-A and F-C have recorded runs (MK-3)");
}

#[test]
fn larkspur_samples_are_complete() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let spec = std::fs::read_to_string(root.join("apis/larkspur.openapi.json")).unwrap();
    let ops = il_connectors::openapi::spec::parse(&spec).unwrap();
    let mut ids: Vec<_> = ops.iter().map(|o| (o.id.as_str(), o.risky)).collect();
    ids.sort();
    assert_eq!(
        ids,
        [("delete_customer", true), ("list_invoices", false), ("lookup_customer", false), ("open_ticket", false)]
    );
    let mock: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("apis/larkspur.mock.json")).unwrap()).unwrap();
    for op in ops.iter().filter(|o| !o.risky) {
        assert!(
            mock["responses"][&op.id]["body"].is_object() || mock["responses"][&op.id]["body"].is_array(),
            "{}",
            op.id
        );
    }
    for doc in ["refund-policy-2026.md", "plans-and-limits.md", "help-center-2025.md", "support-voice-memo.mp3"] {
        assert!(root.join("docs").join(doc).metadata().unwrap().len() > 100, "{doc}");
    }
}

async fn call(app: &axum::Router, method: &str, path: &str) -> serde_json::Value {
    use tower::ServiceExt;
    let req = axum::http::Request::builder().method(method).uri(path).body(axum::body::Body::empty()).unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert!(resp.status().is_success(), "{method} {path}: {}", resp.status());
    serde_json::from_slice(&axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap()).unwrap()
}

#[tokio::test]
async fn every_template_test_runs_end_to_end_in_fake_mode() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let example = std::fs::read_to_string(root.join("config/interlock.example.toml")).unwrap();
    let mut config = serde_json::to_value(toml::from_str::<toml::Value>(&example).unwrap()).unwrap();
    let data = tempfile::tempdir().unwrap();
    config["server"]["templates_dir"] = root.join("fixtures/templates").to_str().unwrap().into();
    config["storage"]["path"] = data.path().to_str().unwrap().into();
    let app = il_server::app(il_server::build_state(il_server::Config::from_value(config).unwrap()));
    for template in call(&app, "GET", "/api/templates").await.as_array().unwrap() {
        let id = template["id"].as_str().unwrap();
        let flow = call(&app, "POST", &format!("/api/templates/{id}/use")).await;
        let run = call(&app, "POST", &format!("/api/flows/{}/test", flow["id"].as_str().unwrap())).await;
        let mut detail = serde_json::Value::Null;
        for _ in 0..200 {
            detail = call(&app, "GET", &format!("/api/runs/{}", run["runId"].as_str().unwrap())).await;
            if !["queued", "running"].contains(&detail["run"]["status"].as_str().unwrap()) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
        let status = detail["run"]["status"].as_str().unwrap();
        assert!(["succeeded", "waiting"].contains(&status), "{id}: {status} {}", detail["run"]["error"]);
        let ran = detail["nodeRuns"].as_array().unwrap();
        assert!(!ran.is_empty(), "{id} ran its steps, not an empty run");
        assert!(ran.iter().all(|n| n["status"] != "failed"), "{id}: {ran:?}");
    }
}
