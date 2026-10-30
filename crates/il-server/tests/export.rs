//! Export as code (Q36): a template flow exported from one server runs from its bundle on another,
//! offline (`IL_PROVIDERS=fake`), behind the same OpenAI-compatible API, and no secret leaves.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use il_sdk::services::store::Credential;
use serde_json::{Value, json};
use tower::ServiceExt;

const SECRET: &str = "sk-live-hosted-secret-value";
const API_KEY: &str = "self-hosted-key-0123456789-abcdefghij";

fn example_config(root: &Path) -> Value {
    let example = std::fs::read_to_string(root.join("config/interlock.example.toml")).unwrap();
    let mut config = serde_json::to_value(toml::from_str::<toml::Value>(&example).unwrap()).unwrap();
    config["server"]["templates_dir"] = root.join("fixtures/templates").to_str().unwrap().into();
    config
}

async fn send(app: &axum::Router, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let response = app.clone().oneshot(request).await.unwrap();
    let (parts, body) = response.into_parts();
    (parts.status, parts.headers, axum::body::to_bytes(body, usize::MAX).await.unwrap().to_vec())
}

fn chat(key: &str) -> Request<Body> {
    let body =
        json!({"model": "larkspur-support-agent", "messages": [{"role": "user", "content": "How do refunds work?"}]});
    Request::post("/v1/chat/completions")
        .header(header::AUTHORIZATION, format!("Bearer {key}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// Writes the zip's files under `dir`, without the archive's top folder.
fn unzip(bytes: Vec<u8>, dir: &Path) -> Vec<String> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut paths = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let name = file.name().to_owned();
        let (root, path) = name.split_once('/').unwrap();
        assert_eq!(root, "larkspur-support-agent", "one top folder");
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains(SECRET), "{path} leaks the connection's secret");
        let target = dir.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
        paths.push(path.to_owned());
    }
    paths
}

#[tokio::test]
async fn q36_exported_template_runs_from_its_bundle_offline() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

    // Hosted: the support agent template, its billing agent bound to a saved API connection.
    let hosted = il_server::build_state(il_server::Config::from_value(example_config(&root)).unwrap());
    let app = il_server::app(hosted.clone());
    let (status, _, body) =
        send(&app, Request::post("/api/templates/tpl_support_agent/use").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    let flow_id = serde_json::from_slice::<Value>(&body).unwrap()["id"].as_str().unwrap().to_owned();
    let store = &hosted.services.store;
    let credential = Credential {
        id: "cred_larkspur".into(),
        app: "openapi".into(),
        secret: json!({"api_key": SECRET}),
        meta: json!({"label": "Larkspur API"}),
    };
    store.put_credential("ws_local", &credential).await.unwrap();
    let mut flow = store.get_flow(&flow_id).await.unwrap().unwrap();
    let billing = flow.nodes.iter_mut().find(|n| n.id == "n_billing").unwrap();
    billing.credentials.insert("api".into(), "cred_larkspur".into());
    let instructions = billing.settings["instructions"].clone();
    store.put_flow(&flow).await.unwrap();

    // Export, as the Publish page's button does.
    let (status, headers, zip) =
        send(&app, Request::get(format!("/api/flows/{flow_id}/export")).body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers[header::CONTENT_TYPE], "application/zip");
    assert!(headers[header::CONTENT_DISPOSITION].to_str().unwrap().contains("larkspur-support-agent-interlock.zip"));
    let bundle = tempfile::tempdir().unwrap();
    let paths = unzip(zip, bundle.path());
    for expected in [
        "README.md",
        "INTERLOCK.md",
        ".env.example",
        "compose.yaml",
        "interlock.toml",
        "interlock.bundle.json",
        "prompts/n_billing.instructions.md",
    ] {
        assert!(paths.iter().any(|p| p == expected), "{expected} missing from {paths:?}");
    }
    assert!(paths.iter().any(|p| p.starts_with("files/") && p.ends_with("larkspur.openapi.json")));
    let env_example = std::fs::read_to_string(bundle.path().join(".env.example")).unwrap();
    assert!(env_example.contains("\nLARKSPUR_API_API_KEY=\n"), "{env_example}");

    // Self-hosted: a fresh engine with the bundle's own config, loaded with IL_PROVIDERS=fake.
    let toml_text = std::fs::read_to_string(bundle.path().join("interlock.toml")).unwrap();
    let config = serde_json::to_value(toml::from_str::<toml::Value>(&toml_text).unwrap()).unwrap();
    let runtime = il_server::build_state(il_server::Config::from_value(config).unwrap());
    assert_eq!(runtime.config.providers, "fake");
    let env: HashMap<&str, &str> = HashMap::from([("INTERLOCK_API_KEY", API_KEY), ("LARKSPUR_API_API_KEY", "sk-mine")]);
    let lookup = |name: &str| env.get(name).map(|v| (*v).to_owned());
    il_server::bundle::load(&runtime, bundle.path(), lookup).await.unwrap();
    let local = il_server::app(runtime.clone());

    let (status, _, body) = send(&local, chat(API_KEY)).await;
    assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let reply: Value = serde_json::from_slice(&body).unwrap();
    assert!(reply["choices"][0]["message"]["content"].as_str().is_some_and(|c| !c.is_empty()), "{reply}");
    assert_eq!(send(&local, chat("a-wrong-key-that-is-long-enough-to-try")).await.0, StatusCode::UNAUTHORIZED);
    let (status, _, _) = send(&local, Request::get("/api/flows").body(Body::empty()).unwrap()).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED, "headless by default");

    // The flow came back whole: prompts filled in, its API spec stored here, the connection from env.
    let rt_store = &runtime.services.store;
    let loaded = rt_store.get_flow(&flow_id).await.unwrap().unwrap();
    let billing = loaded.node("n_billing").unwrap();
    assert_eq!(billing.settings["instructions"], instructions);
    let spec = billing.settings["apiTools"]["spec"].as_str().unwrap();
    assert!(runtime.services.files.get(spec).await.is_ok(), "the spec file id points at the stored copy");
    let saved = rt_store.credential("ws_local", "cred_larkspur").await.unwrap().unwrap();
    assert_eq!(saved.secret, json!({"api_key": "sk-mine"}));
    assert!(rt_store.is_flow_active(&flow_id).await.unwrap());

    // Loading again (a restart) changes nothing.
    il_server::bundle::load(&runtime, bundle.path(), lookup).await.unwrap();
    assert_eq!(rt_store.get_flow(&flow_id).await.unwrap().unwrap().version, loaded.version);
}
