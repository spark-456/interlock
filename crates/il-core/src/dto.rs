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

/// Metadata returned when listing published API keys; secrets are omitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApiKeySummary {
    pub id: String,
    pub name: String,
    pub revoked: bool,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// `POST /api/keys` input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateApiKeyRequest {
    pub name: String,
}

/// `POST /api/keys` response; `key` is revealed only here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateApiKeyResponse {
    pub id: String,
    pub key: String,
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
    /// The flow as the editor has it, unsaved edits included; the saved flow when absent.
    #[serde(default)]
    #[ts(optional)]
    pub flow: Option<crate::Flow>,
    /// Ids of the steps the person pointed at; the change is about them.
    #[serde(default)]
    pub focus: Vec<String>,
}

/// Response of Ask Interlock: a JSON-Patch diff to accept or discard (AR-8).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AskDiff {
    /// e.g. "+1 step, 1 changed".
    pub summary: String,
    pub json_patch: Json,
    /// What the change does, in the model's words.
    #[serde(default)]
    pub notes: Vec<String>,
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

/// `POST /api/connections`: saves a key-based connection (API key, bot token, app password, webhook).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateConnectionRequest {
    pub app: String,
    /// Shown in lists, e.g. "Support bot".
    #[serde(default)]
    #[ts(optional)]
    pub label: Option<String>,
    /// Secret fields the app's steps read (`token`, `url`, `identifier` + `app_password`…). Encrypted at rest, never returned.
    pub secret: std::collections::BTreeMap<String, String>,
}

/// `GET /api/usage`: the workspace's spend over the last 30 days and the caps in force.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageReport {
    pub total_usd: f64,
    /// One entry per day, oldest first, including days with no spend.
    pub days: Vec<UsageDay>,
    /// Flows that spent anything, most expensive first.
    pub flows: Vec<UsageFlow>,
    /// Spend by model role and other metered work, highest cost first.
    #[serde(default)]
    pub roles: Vec<UsageRole>,
    pub caps: UsageCaps,
}

/// Spend attributed to one model role or other work type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageRole {
    pub role: String,
    pub cost_usd: f64,
}

/// Spend on one UTC day (`YYYY-MM-DD`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageDay {
    pub day: String,
    pub cost_usd: f64,
}

/// Spend by one flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageFlow {
    pub flow_id: String,
    pub name: String,
    pub cost_usd: f64,
}

/// Spending caps in USD.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UsageCaps {
    pub run_usd: f64,
    pub flow_daily_usd: f64,
    pub workspace_daily_usd: f64,
}

/// `GET /api/flows/:id/hooks`: where each webhook step of a flow listens.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WebhookUrl {
    pub node_id: String,
    /// `/hooks/<flowId>/<token>`, relative to the server's public address.
    pub path: String,
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
}

/// `GET /api/auth/providers`: a way this server can sign people in.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct AuthProvider {
    /// `google` or `github`; sign in at `/api/auth/<id>/start`.
    pub id: String,
    pub label: String,
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
