//! Templates: checked-in Flow JSON under `templates_dir` (`<id>.json`), listed for the gallery and
//! copied into a workspace on use or when a demo workspace starts.

use std::collections::BTreeSet;
use std::path::{Path as FsPath, PathBuf};

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use il_core::Flow;
use il_core::dto::TemplateSummary;

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

/// A copy of `flow` for `workspace_id`, with a fresh id, not yet saved.
pub(super) fn instance(mut flow: Flow, workspace_id: &str) -> Flow {
    flow.id = il_sdk::id("flw");
    flow.workspace_id = workspace_id.into();
    flow.version = 0;
    flow
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
    let flow = instance(read(&path).await?, &ws.0.workspace_id);
    state.services.store.put_flow(&flow).await.map_err(ApiErr::internal)?;
    Ok(Json(flow))
}
