//! HTTP API DTOs (PRD §8.5). Request/response bodies that are not domain types themselves.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{FlowKind, Json, NodeManifest, RunStatus};

/// `GET /api/nodes`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct CatalogResponse {
    pub nodes: Vec<NodeManifest>,
}

/// Row in `GET /api/flows`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FlowSummary {
    pub id: String,
    pub name: String,
    pub kind: FlowKind,
    pub version: u32,
    pub active: bool,
    pub updated_at: DateTime<Utc>,
    pub last_run: Option<RunSummary>,
    /// Apps used by the flow (logo slots).
    pub apps: Vec<String>,
}

/// Row in `GET /api/flows/:id/runs`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunSummary {
    pub id: String,
    pub status: RunStatus,
    pub started_at: DateTime<Utc>,
    #[ts(type = "number | null")]
    pub ms: Option<u64>,
    pub cost_usd: f64,
    pub replay: bool,
}

/// `POST /api/builds`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BuildRequest {
    pub goal: String,
    #[serde(default)]
    pub kind: Option<FlowKind>,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Optional app/API hints (`gmail`, `slack`, an OpenAPI URL…).
    #[serde(default)]
    pub hints: Vec<String>,
}

/// A file, link or audio clip attached to a build goal (AR-1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Attachment {
    pub kind: AttachmentKind,
    pub name: String,
    /// Set for uploaded files and audio.
    #[serde(default)]
    pub file_id: Option<String>,
    /// Set for links.
    #[serde(default)]
    pub url: Option<String>,
    pub role: AttachmentRole,
}

/// Attachment kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AttachmentKind {
    File,
    Link,
    Audio,
    Api,
}

/// Whether an attachment is reference knowledge or instructions for the assistant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum AttachmentRole {
    Knowledge,
    Instructions,
}

/// Response of `POST /api/builds`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BuildCreated {
    pub build_id: String,
}

/// `POST /api/builds/:id/answers`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BuildAnswers {
    /// Question id → answer.
    pub answers: std::collections::BTreeMap<String, String>,
}

/// `POST /api/flows/:id/ask`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct AskRequest {
    pub message: String,
}

/// Response of Ask Interlock: a JSON-Patch diff to accept or discard (AR-8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AskDiff {
    /// e.g. "+1 step, 1 changed".
    pub summary: String,
    pub json_patch: Json,
}

/// `POST /api/flows/:id/test` and `POST /api/flows/:id/nodes/:nodeId/run`
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct TestRunRequest {
    /// Sample input items; empty means "use the trigger's sample".
    #[serde(default)]
    pub items: Vec<Json>,
}

/// Response with a new run id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunCreated {
    pub run_id: String,
}

/// Row in `GET /api/templates`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TemplateSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: FlowKind,
    pub apps: Vec<String>,
    pub has_recorded_run: bool,
}

/// Row in `GET /api/connections`. Tokens are never returned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConnectionSummary {
    pub id: Option<String>,
    pub app: String,
    pub label: String,
    pub status: ConnectionStatus,
    pub account: Option<String>,
}

/// Connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ConnectionStatus {
    Connected,
    NotConnected,
    Expired,
}

/// `GET /api/me`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Me {
    pub user: Option<UserSummary>,
    pub workspace_id: String,
    pub demo: bool,
}

/// Signed-in user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UserSummary {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// `GET /healthz`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Health {
    pub ok: bool,
    pub version: String,
}
