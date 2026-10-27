//! Run and build events (PRD §8.4). Ordered per stream, persisted, served over SSE.
//!
//! Wire shape: `{ seq, ts, runId | buildId, nodeId?, type, data }`. The UI ignores unknown types.

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{Item, Json};

/// One event in a run stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RunEvent {
    #[ts(type = "number")]
    pub seq: u64,
    pub ts: DateTime<Utc>,
    pub run_id: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(flatten)]
    pub kind: RunEventKind,
}

/// Run event payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum RunEventKind {
    #[serde(rename = "run.started")]
    RunStarted {
        #[serde(rename = "flowId")]
        flow_id: String,
        trigger: String,
    },
    #[serde(rename = "node.started")]
    NodeStarted {},
    #[serde(rename = "node.progress")]
    NodeProgress { pct: f32, message: String },
    /// Preview of output items (max 3) on a port.
    #[serde(rename = "node.items")]
    NodeItems {
        port: String,
        items: Vec<Item>,
        #[ts(type = "number")]
        total: u64,
    },
    #[serde(rename = "node.succeeded")]
    NodeSucceeded {
        #[ts(type = "number")]
        count: u64,
        #[ts(type = "number")]
        ms: u64,
        #[serde(rename = "costUsd")]
        cost_usd: f64,
    },
    #[serde(rename = "node.failed")]
    NodeFailed { error: String, retryable: bool },
    #[serde(rename = "node.waiting")]
    NodeWaiting {
        #[serde(rename = "approvalId")]
        approval_id: String,
    },
    #[serde(rename = "approval.decided")]
    ApprovalDecided {
        #[serde(rename = "approvalId")]
        approval_id: String,
        decision: crate::Decision,
        by: Option<String>,
    },
    #[serde(rename = "run.cap_reached")]
    RunCapReached {
        cap: String,
        #[serde(rename = "limitUsd")]
        limit_usd: f64,
        #[serde(rename = "spentUsd")]
        spent_usd: f64,
    },
    #[serde(rename = "run.succeeded")]
    RunSucceeded {
        #[ts(type = "number")]
        ms: u64,
        #[serde(rename = "costUsd")]
        cost_usd: f64,
    },
    #[serde(rename = "run.failed")]
    RunFailed { error: String },
    #[serde(rename = "run.cancelled")]
    RunCancelled {},
}

/// One event in a build (architect session) stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BuildEvent {
    #[ts(type = "number")]
    pub seq: u64,
    pub ts: DateTime<Utc>,
    pub build_id: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(flatten)]
    pub kind: BuildEventKind,
}

/// Build event payloads.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum BuildEventKind {
    /// `stage` is one of the architect stages (e.g. `understand`, `knowledge`, `tools`, `assemble`, `test`, `ready`).
    #[serde(rename = "build.stage")]
    Stage { stage: String, status: StageStatus },
    #[serde(rename = "build.log")]
    Log { text: String },
    #[serde(rename = "build.question")]
    Question { questions: Vec<BuildQuestion> },
    /// RFC 6902 JSON Patch against the flow being built; nodes and edges appear incrementally.
    #[serde(rename = "build.flow_patch")]
    FlowPatch {
        #[serde(rename = "jsonPatch")]
        json_patch: Json,
    },
    #[serde(rename = "build.connection_needed")]
    ConnectionNeeded { app: String },
    #[serde(rename = "build.test_result")]
    TestResult { passed: u32, failed: u32 },
    #[serde(rename = "build.fix")]
    Fix { description: String },
    #[serde(rename = "build.ready")]
    Ready {
        #[serde(rename = "flowId")]
        flow_id: String,
    },
    #[serde(rename = "build.failed")]
    Failed { error: String },
}

/// Status of a build stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum StageStatus {
    Pending,
    Active,
    Done,
    Failed,
}

/// A clarifying question from the architect (AR-2).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct BuildQuestion {
    pub id: String,
    pub text: String,
    pub options: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_event_wire_shape() {
        let ev = RunEvent {
            seq: 4,
            ts: DateTime::parse_from_rfc3339("2026-10-27T09:00:00Z").unwrap().into(),
            run_id: "run_1".into(),
            node_id: Some("n_script".into()),
            kind: RunEventKind::NodeSucceeded { count: 3, ms: 1200, cost_usd: 0.01 },
        };
        let v = serde_json::to_value(&ev).unwrap();
        assert_eq!(v["type"], "node.succeeded");
        assert_eq!(v["data"]["costUsd"], 0.01);
        assert_eq!(v["runId"], "run_1");
        let back: RunEvent = serde_json::from_value(v).unwrap();
        assert_eq!(back, ev);
    }

    #[test]
    fn build_event_wire_shape() {
        let v = serde_json::json!({
            "seq": 1, "ts": "2026-10-27T09:00:00Z", "buildId": "bld_1",
            "type": "build.stage", "data": {"stage": "understand", "status": "active"}
        });
        let ev: BuildEvent = serde_json::from_value(v).unwrap();
        assert!(matches!(ev.kind, BuildEventKind::Stage { status: StageStatus::Active, .. }));
        let empty = serde_json::json!({"seq": 2, "ts": "2026-10-27T09:00:00Z", "runId": "r", "type": "run.cancelled", "data": {}});
        let ev: RunEvent = serde_json::from_value(empty).unwrap();
        assert_eq!(ev.kind, RunEventKind::RunCancelled {});
    }
}
