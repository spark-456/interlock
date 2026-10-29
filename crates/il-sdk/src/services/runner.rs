//! Starting, resuming and cancelling runs. Implemented by `il-engine`; used by triggers,
//! the architect's test runs, publish (chat/API/MCP) and the server.

use async_trait::async_trait;
use il_core::{ApprovalDecision, Item, Run};
use serde::{Deserialize, Serialize};

/// Runner errors.
#[derive(Debug, thiserror::Error)]
pub enum RunnerError {
    #[error("flow not found: {0}")]
    FlowNotFound(String),
    #[error("run not found: {0}")]
    RunNotFound(String),
    #[error("invalid flow: {0}")]
    InvalidFlow(String),
    #[error("cap reached: {0}")]
    CapReached(String),
    #[error("{0}")]
    Other(String),
}

/// How a run starts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartRun {
    pub flow_id: String,
    /// Specific version; `None` = latest.
    pub version: Option<u32>,
    /// Trigger type id, or `test`, `api`, `replay`.
    pub trigger: String,
    /// Items emitted by the trigger node.
    pub items: Vec<Item>,
    /// Start at this trigger node (flows may have several).
    pub trigger_node: Option<String>,
    /// Run only this step, with `items` as its input (ED-7 "Run this step").
    #[serde(default)]
    pub only: Option<String>,
}

/// The Respond node's output for chat/API/MCP invocations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvokeOutcome {
    pub run: Run,
    pub response: Vec<Item>,
}

/// Run control.
#[async_trait]
pub trait FlowRunner: Send + Sync {
    /// Starts a run in the background; returns the run id.
    async fn start(&self, req: StartRun) -> Result<String, RunnerError>;
    /// Starts a run and waits until it finishes or pauses; returns Respond output.
    async fn invoke(&self, req: StartRun) -> Result<InvokeOutcome, RunnerError>;
    /// Applies a human decision and resumes the paused run.
    async fn decide(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        by: Option<String>,
    ) -> Result<(), RunnerError>;
    /// Cancels a run.
    async fn cancel(&self, run_id: &str) -> Result<(), RunnerError>;
    /// Replays a recorded run's events into a new run id (EN-8).
    async fn replay(&self, recorded_run_id: &str) -> Result<String, RunnerError>;
}
