//! Templates: checked-in Flow JSON under `templates_dir` (`<id>.json`), listed for the gallery and
//! copied into a workspace on use (or all at once for offline builds with fake providers).

use std::collections::BTreeSet;
use std::path::{Path as FsPath, PathBuf};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::dto::TemplateSummary;
use il_core::events::{RunEvent, RunEventKind};
use il_core::{Flow, Run, RunStatus};
use serde_json::Value;

use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/templates", get(list)).route("/api/templates/{id}/use", post(use_template))
}

/// Every template as `(id, flow)`, sorted by id. A missing directory means no templates.
pub(super) async fn load_all(dir: &str) -> ApiResult<Vec<(String, Flow)>> {
    let mut entries = match tokio::fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(ApiErr::internal(error)),
    };
    let mut out = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(ApiErr::internal)? {
        let path = entry.path();
        if let Some(id) = template_id(&path) {
            out.push((id, read(&path).await?));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

/// Copies every template into a workspace, with its recorded run when it has one (MK-3).
pub(super) async fn seed(state: &AppState, workspace_id: &str) -> ApiResult<()> {
    let dir = &state.config.server.templates_dir;
    for (id, flow) in load_all(dir).await? {
        let flow = instance(state, flow, workspace_id).await?;
        state.services.store.put_flow(&flow).await.map_err(ApiErr::internal)?;
        if let Ok(bytes) = tokio::fs::read(recorded(dir, &id)).await {
            import_run(state, &flow, &bytes).await?;
        }
    }
    Ok(())
}

/// Stores a recorded run (its run events) as a finished replay run of `flow`, so the flow's
/// history shows it and the run view replays it.
async fn import_run(state: &AppState, flow: &Flow, bytes: &[u8]) -> ApiResult<()> {
    let mut events: Vec<RunEvent> = serde_json::from_slice(bytes).map_err(ApiErr::internal)?;
    let (Some(first), Some(last)) = (events.first(), events.last()) else { return Ok(()) };
    let (status, cost_usd, error) = match &last.kind {
        RunEventKind::RunSucceeded { cost_usd, .. } => (RunStatus::Succeeded, *cost_usd, None),
        RunEventKind::RunFailed { error } => (RunStatus::Failed, 0.0, Some(error.clone())),
        _ => (RunStatus::Succeeded, 0.0, None),
    };
    let trigger = match &first.kind {
        RunEventKind::RunStarted { trigger, .. } => trigger.clone(),
        _ => "replay".into(),
    };
    let run = Run {
        id: il_sdk::id("run"),
        flow_id: flow.id.clone(),
        workspace_id: flow.workspace_id.clone(),
        flow_version: flow.version,
        status,
        trigger,
        started_at: first.ts,
        finished_at: Some(last.ts),
        cost_usd,
        error,
        replay: true,
        state: Value::Null,
    };
    let store = &state.services.store;
    store.put_run(&run).await.map_err(ApiErr::internal)?;
    for event in &mut events {
        event.run_id = run.id.clone();
        if let RunEventKind::RunStarted { flow_id, .. } = &mut event.kind {
            *flow_id = flow.id.clone();
        }
        store.append_run_event(event).await.map_err(ApiErr::internal)?;
    }
    Ok(())
}

/// A copy of `flow` for `workspace_id`, with a fresh id, not yet saved.
async fn instance(state: &AppState, mut flow: Flow, workspace_id: &str) -> ApiResult<Flow> {
    flow.id = il_sdk::id("flw");
    flow.workspace_id = workspace_id.into();
    flow.version = 0;
    attach_sample_files(state, &mut flow).await?;
    for node in &mut flow.nodes {
        if node.settings["apiTools"]["spec"] == "@fixture/larkspur.openapi.json" {
            let path = FsPath::new(&state.config.server.templates_dir)
                .parent()
                .unwrap_or(FsPath::new("."))
                .join("apis/larkspur.openapi.json");
            let bytes = tokio::fs::read(path).await.map_err(ApiErr::internal)?;
            let file = state
                .services
                .files
                .put_for_workspace(workspace_id, "larkspur.openapi.json", "application/json", bytes.into())
                .await
                .map_err(ApiErr::internal)?;
            node.settings["apiTools"]["spec"] = Value::String(file.id);
        }
    }
    Ok(flow)
}

/// A trigger's sample items name their files (`"files": {"attachment": "invoice.txt"}`); each
/// is stored from `<templates_dir>/../samples/` and replaced by its file reference.
async fn attach_sample_files(state: &AppState, flow: &mut Flow) -> ApiResult<()> {
    let dir = FsPath::new(&state.config.server.templates_dir).parent().unwrap_or(FsPath::new(".")).join("samples");
    for node in &mut flow.nodes {
        let Some(samples) = node.settings.get_mut("sampleItems").and_then(Value::as_array_mut) else { continue };
        for files in samples.iter_mut().filter_map(|s| s.get_mut("files").and_then(Value::as_object_mut)) {
            for file in files.values_mut() {
                let Some(name) = file.as_str().filter(|n| !n.contains(['/', '\\']) && !n.starts_with('.')) else {
                    continue;
                };
                let bytes = tokio::fs::read(dir.join(name)).await.map_err(ApiErr::internal)?;
                let mime = mime_guess::from_path(name).first_or_octet_stream().to_string();
                let stored = state
                    .services
                    .files
                    .put_for_workspace(&flow.workspace_id, name, &mime, bytes.into())
                    .await
                    .map_err(ApiErr::internal)?;
                *file = serde_json::to_value(stored).map_err(ApiErr::internal)?;
            }
        }
    }
    Ok(())
}

fn template_id(path: &FsPath) -> Option<String> {
    (path.extension()? == "json").then(|| path.file_stem()?.to_str().map(String::from))?
}

async fn read(path: &FsPath) -> ApiResult<Flow> {
    let bytes = tokio::fs::read(path).await.map_err(ApiErr::internal)?;
    serde_json::from_slice(&bytes).map_err(ApiErr::internal)
}

/// Recorded runs sit beside the templates: `<templates_dir>/../recorded/<id>.json`.
fn recorded(dir: &str, id: &str) -> PathBuf {
    FsPath::new(dir).parent().unwrap_or(FsPath::new(".")).join("recorded").join(format!("{id}.json"))
}

async fn list(State(state): State<AppState>, _ws: Ws) -> ApiResult<Json<Vec<TemplateSummary>>> {
    let dir = &state.config.server.templates_dir;
    let catalog = il_sdk::registry::catalog();
    let apps_of = |flow: &Flow| -> Vec<String> {
        let apps: BTreeSet<_> = flow
            .nodes
            .iter()
            .filter_map(|n| catalog.iter().find(|m| m.type_id == n.node_type).and_then(|m| m.app.clone()))
            .collect();
        apps.into_iter().collect()
    };
    let mut out = Vec::new();
    for (id, flow) in load_all(dir).await? {
        out.push(TemplateSummary {
            has_recorded_run: tokio::fs::try_exists(recorded(dir, &id)).await.unwrap_or(false),
            description: flow.meta.goal.clone().unwrap_or_default(),
            apps: apps_of(&flow),
            name: flow.name,
            kind: flow.kind,
            id,
        });
    }
    Ok(Json(out))
}

async fn use_template(State(state): State<AppState>, ws: Ws, Path(id): Path<String>) -> ApiResult<Json<Flow>> {
    let dir = &state.config.server.templates_dir;
    let path = FsPath::new(dir).join(format!("{id}.json"));
    if template_id(&path).as_deref() != Some(id.as_str()) || !tokio::fs::try_exists(&path).await.unwrap_or(false) {
        return Err(ApiErr::not_found("Template"));
    }
    let flow = instance(&state, read(&path).await?, &ws.0.workspace_id).await?;
    state.services.store.put_flow(&flow).await.map_err(ApiErr::internal)?;
    Ok(Json(flow))
}
