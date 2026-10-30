//! Runs an exported bundle (Q36, `docs/EXPORT.md`): with `IL_BUNDLE=<folder>` the server loads the
//! bundle's flows, files, connections and keys into its local workspace at startup and turns the
//! flows on. Loading again changes nothing unless the bundle or `.env` changed.

use std::collections::BTreeMap;
use std::path::{Component, Path};

use anyhow::Context;
use il_core::Flow;
use il_publish::export::manifest::{self, Manifest};
use il_publish::export::{resolve_prompts, rewrite_ids};
use il_publish::keys::KeyManager;
use il_sdk::services::knowledge::{BaseCopy, SourceInput};
use il_sdk::services::store::Credential;
use serde_json::{Value, json};

use crate::wiring::AppState;

/// The self-hosted workspace: the one requests without a session use, so the local app shows it too.
pub const WORKSPACE: &str = "ws_local";
const API_KEY: &str = "INTERLOCK_API_KEY";
const WEBHOOK_TOKEN: &str = "INTERLOCK_WEBHOOK_TOKEN";

/// Loads the bundle in `dir`; `env` reads a setting (the process environment, or a map in tests).
pub async fn load(state: &AppState, dir: &Path, env: impl Fn(&str) -> Option<String>) -> anyhow::Result<()> {
    let env = |name: &str| env(name).filter(|v| !v.trim().is_empty());
    let manifest: Manifest = serde_json::from_slice(&read(dir, manifest::PATH)?)
        .with_context(|| format!("{} is not a valid bundle manifest", manifest::PATH))?;
    anyhow::ensure!(
        manifest.format <= manifest::FORMAT,
        "this bundle needs a newer Interlock engine (bundle format {})",
        manifest.format
    );
    let store = &state.services.store;

    let mut ids = store_files(state, dir, &manifest).await?;
    let files = ids.len();
    ids.extend(rebuild_knowledge(state, dir, &manifest).await?);

    let mut flow_ids = Vec::new();
    for path in &manifest.flows {
        let mut flow: Flow =
            serde_json::from_slice(&read(dir, path)?).with_context(|| format!("{path} is not a valid flow"))?;
        resolve_prompts(&mut flow, |p| read(dir, p).ok().and_then(|b| String::from_utf8(b).ok()))
            .map_err(anyhow::Error::msg)?;
        rewrite_ids(&mut flow, &ids);
        flow.workspace_id = WORKSPACE.into();
        let saved = store.get_flow(&flow.id).await?;
        if saved.as_ref().is_none_or(|s| *s != Flow { version: s.version, ..flow.clone() }) {
            let version = store.put_flow(&flow).await?;
            tracing::info!(flow_id = %flow.id, version, "bundle flow saved");
        }
        flow_ids.push(flow.id);
    }

    let mut missing = Vec::new();
    for c in &manifest.connections {
        let values: Vec<_> = c.fields.iter().map(|f| (f, env(&f.env))).collect();
        missing.extend(values.iter().filter(|(_, v)| v.is_none()).map(|(f, _)| f.env.clone()));
        if values.iter().any(|(_, v)| v.is_none()) {
            continue;
        }
        let mut secret: serde_json::Map<String, Value> =
            values.into_iter().map(|(f, v)| (f.key.clone(), json!(v))).collect();
        if c.oauth {
            // Expired on purpose: the first step that uses it fetches an access token.
            secret.insert("access_token".into(), json!(""));
            secret.insert("expires_at".into(), json!(0));
        }
        let credential = Credential {
            id: c.id.clone(),
            app: c.app.clone(),
            secret: Value::Object(secret),
            meta: json!({"label": c.label, "kind": "env"}),
        };
        store
            .put_credential(WORKSPACE, &credential)
            .await
            .with_context(|| format!("saving the {} connection (is IL_MASTER_KEY set?)", c.label))?;
    }
    if !missing.is_empty() {
        tracing::warn!(?missing, "connections not set in .env; the steps that use them fail until they are");
    }

    let keys = KeyManager::new(store.clone());
    let previous = store.kv_get("bundle:api_key").await?.and_then(|v| v.as_str().map(str::to_owned));
    let current = match env(API_KEY) {
        Some(raw) => {
            let id = keys.import(WORKSPACE, API_KEY, &raw).await?;
            Some(id.with_context(|| format!("{API_KEY} needs at least 32 characters"))?)
        }
        None => {
            tracing::warn!("{API_KEY} is not set: the API and MCP endpoints refuse every call");
            None
        }
    };
    if let Some(old) = previous.filter(|old| Some(old) != current.as_ref()) {
        keys.revoke(WORKSPACE, &old).await.ok();
    }
    store.kv_put("bundle:api_key", &json!(current)).await?;

    if !manifest.webhooks.is_empty() {
        match env(WEBHOOK_TOKEN) {
            Some(token) => {
                anyhow::ensure!(
                    token.len() >= 24 && token.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                    "{WEBHOOK_TOKEN} needs 24 or more letters, digits, `-` or `_`"
                );
                for hook in &manifest.webhooks {
                    let key = il_nodes::trigger::webhook::token_key(&hook.flow_id, &hook.node_id);
                    store.kv_put(&key, &json!(token)).await?;
                }
            }
            None => tracing::warn!("{WEBHOOK_TOKEN} is not set: webhooks get a token nobody knows"),
        }
    }

    for id in &flow_ids {
        if let Err(error) = state.triggers.activate(id).await {
            tracing::error!(flow_id = %id, %error, "bundle flow could not be turned on");
        }
    }
    tracing::info!(flows = flow_ids.len(), files, "bundle loaded from {}", dir.display());
    Ok(())
}

/// Stores the bundle's files once each; returns old id → stored id.
async fn store_files(state: &AppState, dir: &Path, manifest: &Manifest) -> anyhow::Result<BTreeMap<String, String>> {
    let store = &state.services.store;
    let mut ids = BTreeMap::new();
    for file in &manifest.files {
        let key = format!("bundle:file:{}", file.id);
        let known = store.kv_get(&key).await?.and_then(|v| v.as_str().map(str::to_owned));
        let id = match known {
            Some(id) if state.services.files.get(&id).await.is_ok() => id,
            _ => {
                let bytes = read(dir, &file.path)?;
                let stored =
                    state.services.files.put_for_workspace(WORKSPACE, &file.name, &file.mime, bytes.into()).await?;
                store.kv_put(&key, &json!(stored.id)).await?;
                stored.id
            }
        };
        ids.insert(file.id.clone(), id);
    }
    Ok(ids)
}

/// Rebuilds each saved knowledge base from its sources' text once (embedding them with the configured
/// model); returns old id → rebuilt id.
async fn rebuild_knowledge(
    state: &AppState,
    dir: &Path,
    manifest: &Manifest,
) -> anyhow::Result<BTreeMap<String, String>> {
    let (store, knowledge) = (&state.services.store, &state.services.knowledge);
    let mut ids = BTreeMap::new();
    for kb in &manifest.knowledge {
        let key = format!("bundle:kb:{}", kb.id);
        let known = store.kv_get(&key).await?.and_then(|v| v.as_str().map(str::to_owned));
        let id = match known {
            Some(id) if knowledge.owns_base(WORKSPACE, &id).await.unwrap_or(false) => id,
            _ => {
                let copy: BaseCopy = serde_json::from_slice(&read(dir, &kb.path)?)
                    .with_context(|| format!("{} is not a valid knowledge base", kb.path))?;
                let id = knowledge.create_base(WORKSPACE, &copy.name).await?;
                let sources = copy
                    .sources
                    .into_iter()
                    .map(|s| match s.role.as_str() {
                        "instructions" => SourceInput::InstructionsText { title: s.title, text: s.text },
                        _ => SourceInput::Text { title: s.title, text: s.text },
                    })
                    .collect();
                knowledge.add_sources(&id, sources).await.with_context(|| format!("rebuilding {}", kb.name))?;
                store.kv_put(&key, &json!(id)).await?;
                tracing::info!(kb_id = %id, name = %kb.name, "bundle knowledge base rebuilt");
                id
            }
        };
        ids.insert(kb.id.clone(), id);
    }
    Ok(ids)
}

/// A file inside the bundle; paths that leave the folder are refused.
fn read(dir: &Path, path: &str) -> anyhow::Result<Vec<u8>> {
    let relative = Path::new(path);
    anyhow::ensure!(
        relative.components().all(|c| matches!(c, Component::Normal(_))),
        "{path} is outside the bundle folder"
    );
    std::fs::read(dir.join(relative)).with_context(|| format!("reading {path} from the bundle"))
}
