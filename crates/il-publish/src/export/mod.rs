//! Export as code (Q36, `docs/EXPORT.md`): a flow as a folder of plain files that the open-source
//! engine loads with `IL_BUNDLE`, so it runs in the user's own infrastructure with the same API, MCP,
//! widget and webhook endpoints. Connections export their shape only; values come from env.

mod docs;
pub mod manifest;
mod scan;

use std::collections::BTreeMap;
use std::io::Write;

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use il_core::Flow;
use il_core::manifest::CredentialKind;
use il_sdk::Services;
use serde_json::Value;

pub use manifest::Manifest;
pub use scan::{resolve_prompts, rewrite_ids};

use crate::openai::slug;
use manifest::{Connection, Engine, Field, FileSpec, KnowledgeSpec, Webhook};

/// Export errors.
#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("storage: {0}")]
    Store(String),
    #[error("files: {0}")]
    Files(String),
    #[error("zip: {0}")]
    Zip(String),
}

/// One file of a bundle, by its path inside the bundle folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub path: String,
    pub bytes: Vec<u8>,
}

impl Entry {
    fn text(path: &str, text: impl Into<String>) -> Self {
        Self { path: path.into(), bytes: text.into().into_bytes() }
    }
}

/// `[export]` config: where bundles build the engine from.
fn engine(config: &Value) -> Engine {
    let text = |key: &str, default: &str| {
        config.pointer(&format!("/export/{key}")).and_then(Value::as_str).unwrap_or(default).to_owned()
    };
    Engine {
        version: env!("CARGO_PKG_VERSION").into(),
        source: text("source", "https://github.com/spark-456/interlock.git"),
        git_ref: text("git_ref", "main"),
    }
}

/// The bundle's files for `flow`, which the caller has checked belongs to its workspace. Files the
/// flow refers to are included when the workspace owns them (or nobody does, as for step outputs).
pub async fn bundle(services: &Services, config: &Value, flow: &Flow) -> Result<Vec<Entry>, ExportError> {
    let workspace_id = flow.workspace_id.clone();
    let mut exported = flow.clone();
    exported.workspace_id = String::new();
    let mut entries = Vec::new();

    let mut files = Vec::new();
    for id in scan::ids(&exported, "file_") {
        let owner = services.files.owner(&id).await.map_err(|e| ExportError::Files(e.to_string()))?;
        if owner.is_some_and(|owner| owner != workspace_id) {
            continue;
        }
        // A file deleted since the flow was saved is left out; the step reports it when it runs.
        let Ok((file, bytes)) = services.files.get(&id).await else { continue };
        let path = format!("files/{id}/{}", scan::safe(&file.name));
        entries.push(Entry { path: path.clone(), bytes: bytes.to_vec() });
        files.push(FileSpec { id, path, name: file.name, mime: file.mime });
    }

    let mut knowledge = Vec::new();
    for id in scan::ids(&exported, "kb_") {
        let copy =
            services.knowledge.export_base(&workspace_id, &id).await.map_err(|e| ExportError::Store(e.to_string()))?;
        let Some(copy) = copy else { continue };
        let path = format!("knowledge/{id}.json");
        entries.push(Entry { path: path.clone(), bytes: pretty(&copy)? });
        knowledge.push(KnowledgeSpec { id, name: copy.name, path });
    }

    let connections = connections(services, &workspace_id, &mut exported).await?;
    for (path, text) in scan::take_prompts(&mut exported) {
        entries.push(Entry::text(&path, text));
    }
    let webhooks: Vec<Webhook> = exported
        .nodes
        .iter()
        .filter(|n| n.node_type == "trigger.webhook" && !n.disabled)
        .map(|n| Webhook { flow_id: exported.id.clone(), node_id: n.id.clone(), name: n.name.clone() })
        .collect();

    let flow_path = format!("flows/{}.json", scan::safe(&exported.id));
    let manifest = Manifest {
        format: manifest::FORMAT,
        exported_at: Utc::now(),
        engine: engine(config),
        flows: vec![flow_path.clone()],
        files,
        knowledge,
        env: docs::env_vars(config, &exported, &connections, !webhooks.is_empty()),
        connections,
        webhooks,
    };
    entries.push(Entry { path: flow_path, bytes: pretty(&exported)? });
    entries.push(Entry { path: manifest::PATH.into(), bytes: pretty(&manifest)? });
    entries.push(Entry::text("interlock.toml", docs::config_toml(config).map_err(ExportError::Store)?));
    entries.push(Entry::text(".env.example", docs::env_example(&exported, &manifest)));
    entries.push(Entry::text(".gitignore", ".env\ndata/\n"));
    entries.push(Entry::text("compose.yaml", docs::compose(&exported, &manifest)));
    entries.push(Entry::text("README.md", docs::readme(&exported, &manifest)));
    entries.push(Entry::text("INTERLOCK.md", docs::agent_guide(&exported, &manifest)));
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(entries)
}

/// The connections the flow uses, with env var names for their fields. Stored connections keep their
/// field names (never their values); OAuth ones need only the refresh token. In order of first use.
async fn connections(services: &Services, workspace_id: &str, flow: &mut Flow) -> Result<Vec<Connection>, ExportError> {
    let mut connections: Vec<Connection> = Vec::new();
    for slot in scan::credential_slots(flow) {
        if let Some(existing) = connections.iter_mut().find(|c| c.id == slot.credential_id) {
            if !existing.used_by.contains(&slot.step) {
                existing.used_by.push(slot.step);
            }
            continue;
        }
        let stored = services
            .store
            .credential(workspace_id, &slot.credential_id)
            .await
            .map_err(|e| ExportError::Store(e.to_string()))?;
        let oauth = slot.kind == CredentialKind::OAuth2
            || stored.as_ref().is_some_and(|c| c.secret.get("refresh_token").is_some());
        let fields = match &stored {
            _ if oauth => vec!["refresh_token".to_owned()],
            Some(c) => c.secret.as_object().map(|s| s.keys().cloned().collect()).unwrap_or_default(),
            None => scan::default_fields(slot.kind),
        };
        let app = stored.as_ref().map_or(slot.app.clone(), |c| c.app.clone());
        let label = stored
            .as_ref()
            .and_then(|c| c.meta["label"].as_str().map(str::to_owned))
            .unwrap_or_else(|| slot.label.clone());
        connections.push(Connection {
            id: slot.credential_id,
            app,
            label,
            oauth,
            fields: fields.into_iter().map(|key| Field { key, env: String::new() }).collect(),
            used_by: vec![slot.step],
        });
    }
    // Env names: `<APP>_<FIELD>`; per-step APIs are named after their label; repeats get a number.
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for c in &mut connections {
        let base = if matches!(c.app.as_str(), "openapi" | "http") && c.label != c.app { &c.label } else { &c.app };
        let base = scan::env_name(&[base]);
        let n = seen.entry(base.clone()).and_modify(|n| *n += 1).or_insert(1);
        let prefix = if *n == 1 { base } else { format!("{base}_{n}") };
        for field in &mut c.fields {
            field.env = scan::env_name(&[&prefix, &field.key]);
        }
    }
    Ok(connections)
}

/// `<slug>-interlock.zip`.
pub fn file_name(flow: &Flow) -> String {
    format!("{}-interlock.zip", slug(flow))
}

/// The bundle as a zip, every file under one `<slug>/` folder.
pub fn zip(flow: &Flow, entries: &[Entry]) -> Result<Vec<u8>, ExportError> {
    let root = slug(flow);
    let mut writer = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for entry in entries {
        writer.start_file(format!("{root}/{}", entry.path), options).map_err(|e| ExportError::Zip(e.to_string()))?;
        writer.write_all(&entry.bytes).map_err(|e| ExportError::Zip(e.to_string()))?;
    }
    Ok(writer.finish().map_err(|e| ExportError::Zip(e.to_string()))?.into_inner())
}

fn pretty(value: &impl serde::Serialize) -> Result<Vec<u8>, ExportError> {
    serde_json::to_vec_pretty(value).map_err(|e| ExportError::Store(e.to_string()))
}

/// The bundle as a zip download, or a 500 with the error logged.
pub async fn response(services: &Services, config: &Value, flow: &Flow) -> Response {
    let built = match bundle(services, config, flow).await {
        Ok(entries) => zip(flow, &entries),
        Err(e) => Err(e),
    };
    match built {
        Ok(bytes) => {
            let disposition = format!("attachment; filename=\"{}\"", file_name(flow));
            ([(header::CONTENT_TYPE, "application/zip".to_owned()), (header::CONTENT_DISPOSITION, disposition)], bytes)
                .into_response()
        }
        Err(e) => {
            tracing::error!(flow_id = %flow.id, error = %e, "export failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use il_sdk::prelude::*;
    use il_sdk::services::knowledge::{BaseCopy, SourceText};
    use il_sdk::services::store::Credential;
    use il_sdk::services::{FileStore, Store};
    use il_sdk::testkit::FakeServices;

    use super::*;

    /// A step with a prompt and a required Slack connection, standing in for the real catalog.
    struct ExportNode(NodeManifest);

    impl Default for ExportNode {
        fn default() -> Self {
            Self(
                NodeManifest::new("test.export", Category::Ai, "Export test", "A step with a prompt.")
                    .credential(cred("slack", "slack", CredentialKind::BotToken, "Slack bot token", &[]))
                    .settings_schema(json!({"type": "object", "properties": {
                        "instructions": {"type": "string", "x-widget": "longtext"},
                        "spec": {"type": "string", "x-widget": "file"}}})),
            )
        }
    }

    #[async_trait]
    impl NodeExecutor for ExportNode {
        fn manifest(&self) -> &NodeManifest {
            &self.0
        }
        async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
            Ok(NodeOutput::out(input.main().to_vec()))
        }
    }

    inventory::submit! { NodeFactory::new(|| Box::new(ExportNode::default())) }

    #[tokio::test]
    async fn q36_bundle_keeps_secrets_out_and_lists_every_setting() {
        let fakes = FakeServices::default();
        let services = fakes.services();
        let own = fakes.files.put_for_workspace("ws_a", "faq.md", "text/markdown", "# FAQ".into()).await.unwrap();
        let foreign =
            fakes.files.put_for_workspace("ws_b", "theirs.md", "text/markdown", "private".into()).await.unwrap();
        let secret = "xoxb-REAL-SECRET-VALUE";
        let credential = Credential {
            id: "cred_slack".into(),
            app: "slack".into(),
            secret: json!({"bot_token": secret}),
            meta: json!({"label": "Team Slack"}),
        };
        fakes.store.put_credential("ws_a", &credential).await.unwrap();
        let kb = "kb_01JABCDEFGHJKMNPQRSTVWXYZ0";
        let text = SourceText { title: "Returns".into(), url: None, role: "knowledge".into(), text: "30 days.".into() };
        let copy = BaseCopy { name: "Policies".into(), sources: vec![text] };
        fakes.knowledge.bases.lock().unwrap().insert(kb.into(), copy.clone());
        let flow: Flow = serde_json::from_value(json!({"id": "flw_1", "workspaceId": "ws_a", "version": 4,
            "name": "Help desk", "kind": "app", "edges": [], "nodes": [
                {"id": "n_a", "type": "test.export", "name": "Answer", "credentials": {"slack": "cred_slack"},
                 "settings": {"instructions": "Answer kindly.", "spec": own.id, "knowledgeBases": [kb, "n_kb"]}},
                {"id": "n_b", "type": "test.export", "name": "Escalate",
                 "settings": {"instructions": "Hand off.", "spec": foreign.id}},
                {"id": "n_hook", "type": "trigger.webhook", "name": "Inbound", "settings": {}}]}))
        .unwrap();
        let config = json!({"models": {"provider": "tokenfactory", "api_key": "sk-config-secret",
            "roles": {"agent": "m"}, "fallbacks": [{"provider": "groq", "roles": {"agent": "g"}}]},
            "auth": {"github": {"client_id": "abc"}}, "engine": {"max_parallel": 4}});

        let entries = bundle(&services, &config, &flow).await.unwrap();
        let text = |path: &str| {
            String::from_utf8(entries.iter().find(|e| e.path == path).unwrap_or_else(|| panic!("{path}")).bytes.clone())
                .unwrap()
        };
        for entry in &entries {
            let body = String::from_utf8_lossy(&entry.bytes);
            for leaked in [secret, "sk-config-secret", "private", "ws_a"] {
                assert!(!body.contains(leaked), "{} leaks {leaked}", entry.path);
            }
        }
        assert!(entries.iter().any(|e| e.path == format!("files/{}/faq.md", own.id)));
        assert_eq!(text("prompts/n_a.instructions.md"), "Answer kindly.\n");

        let saved: BaseCopy = serde_json::from_str(&text(&format!("knowledge/{kb}.json"))).unwrap();
        assert_eq!(saved, copy, "a saved knowledge base travels as its sources' text");
        let exported: Flow = serde_json::from_str(&text("flows/flw_1.json")).unwrap();
        assert_eq!(exported.nodes[0].settings["instructions"], "@prompts/n_a.instructions.md");
        assert_eq!(exported.nodes[1].credentials["slack"], "cred_n_b_slack", "a required connection gets an id");

        let env = text(".env.example");
        let assigned: Vec<&str> = env.lines().filter(|l| !l.starts_with('#') && l.contains('=')).collect();
        assert!(assigned.iter().all(|l| l.ends_with('=') || *l == "IL_PROVIDERS=real"), "{assigned:?}");
        for name in [
            "INTERLOCK_API_KEY=",
            "NEBIUS_API_KEY=",
            "# GROQ_API_KEY=",
            "IL_MASTER_KEY=",
            "SLACK_BOT_TOKEN=",
            "SLACK_2_TOKEN=",
            "INTERLOCK_WEBHOOK_TOKEN=",
        ] {
            assert!(env.contains(&format!("\n{name}\n")), "{name} missing from:\n{env}");
        }

        let toml = text("interlock.toml");
        assert!(toml.contains("local_workspace = false") && toml.contains("max_parallel = 4"), "{toml}");
        assert!(!toml.contains("api_key") && !toml.contains("github"), "{toml}");
        let readme = text("README.md");
        for part in [
            "/v1/chat/completions",
            "/mcp/help-desk",
            "/hooks/flw_1/",
            "SLACK_BOT_TOKEN",
            "docker compose up",
            "`knowledge/`",
        ] {
            assert!(readme.contains(part), "README lacks {part}");
        }
        assert!(text("INTERLOCK.md").contains("Never commit `.env`"));

        let zipped = zip(&flow, &entries).unwrap();
        let archive = zip::ZipArchive::new(std::io::Cursor::new(zipped)).unwrap();
        assert!(archive.file_names().any(|n| n == "help-desk/README.md"));
        assert_eq!(archive.len(), entries.len());
    }
}
