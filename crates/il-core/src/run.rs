//! Runs, node runs and approvals (EN-3, EN-7).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Item, Json};

/// One execution of a flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Run {
    pub id: String,
    pub flow_id: String,
    pub workspace_id: String,
    pub flow_version: u32,
    pub status: RunStatus,
    /// Trigger type id, or `test`, `replay`, `api`.
    pub trigger: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub cost_usd: f64,
    pub error: Option<String>,
    /// True when this run is a replay of a recorded run (EN-8).
    #[serde(default)]
    pub replay: bool,
    /// Engine checkpoint used to resume after restart. Opaque to everyone but the engine.
    #[serde(default)]
    pub state: Json,
}

/// Lifecycle of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RunStatus {
    Queued,
    Running,
    Waiting,
    Succeeded,
    Failed,
    Cancelled,
    CapReached,
}

impl RunStatus {
    /// True once the run will never change again.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}

/// Per-node record inside a run (inputs/outputs for the run view).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NodeRun {
    pub run_id: String,
    pub node_id: String,
    pub status: RunStatus,
    pub attempt: u32,
    pub input: Vec<Item>,
    /// Output items by port.
    pub output: std::collections::BTreeMap<String, Vec<Item>>,
    pub error: Option<String>,
    #[ts(type = "number")]
    pub ms: u64,
    pub cost_usd: f64,
}

/// A pending or decided human approval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Approval {
    pub id: String,
    pub workspace_id: String,
    pub flow_id: String,
    pub run_id: String,
    pub node_id: String,
    pub title: String,
    pub message: String,
    /// Editable payload shown to the approver (e.g. post drafts).
    pub payload: Json,
    pub status: ApprovalStatus,
    pub channels: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_timeout: Option<ApprovalTimeout>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decided_by: Option<String>,
}

/// Approval lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Action to take when an approval deadline passes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ApprovalTimeout {
    Approve,
    Reject,
    KeepWaiting,
}

/// Approve or reject.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum Decision {
    Approve,
    Reject,
}

/// Body of `POST /api/approvals/:id/decide`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ApprovalDecision {
    pub decision: Decision,
    /// Edited payload, if the approver changed it.
    #[serde(default)]
    pub payload: Option<Json>,
    #[serde(default)]
    pub comment: Option<String>,
}
