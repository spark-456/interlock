//! Node executor and trigger traits (PRD §8.3).

use std::collections::BTreeMap;

use async_trait::async_trait;
use il_core::{Item, NodeManifest};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ctx::{NodeCtx, TriggerCtx, TriggerHandle};

/// A node implementation. One module per node; registered with `inventory::submit!`.
#[async_trait]
pub trait NodeExecutor: Send + Sync + 'static {
    /// The node's manifest (static per type).
    fn manifest(&self) -> &NodeManifest;

    /// Validates settings before a run. Default: JSON Schema check against the manifest.
    fn validate(&self, settings: &Value) -> Result<(), Vec<ValidationIssue>> {
        crate::validate::default_schema_validate(self.manifest(), settings)
    }

    /// Runs the node. Per-item nodes get one item per call; batch nodes get all items.
    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError>;

    /// Trigger nodes return `Some(self)`; the trigger registry is derived from this.
    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        None
    }
}

/// A node that starts runs (schedule, poll, webhook bind).
#[async_trait]
pub trait TriggerNode: NodeExecutor {
    /// Starts listening; the handle stops it.
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError>;
}

/// Items arriving on each input port.
#[derive(Debug, Clone, Default)]
pub struct NodeInput {
    pub ports: BTreeMap<String, Vec<Item>>,
}

impl NodeInput {
    /// Input with items on the default `in` port.
    pub fn items(items: Vec<Item>) -> Self {
        let mut ports = BTreeMap::new();
        ports.insert("in".to_string(), items);
        Self { ports }
    }

    /// Items on `in` (empty if none).
    pub fn main(&self) -> &[Item] {
        self.port("in")
    }

    /// Items on a named port (empty if none).
    pub fn port(&self, name: &str) -> &[Item] {
        self.ports.get(name).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// Items produced on each output port, plus accounting.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NodeOutput {
    pub ports: BTreeMap<String, Vec<Item>>,
    /// Set when the node paused for a human decision (from `ctx.request_approval`).
    pub waiting_for: Option<String>,
    /// Extra cost not already metered through the gateway (e.g. media providers).
    pub cost_usd: f64,
}

impl NodeOutput {
    /// Items on the default `out` port.
    pub fn out(items: Vec<Item>) -> Self {
        Self::port("out", items)
    }

    /// Items on a named port.
    pub fn port(name: &str, items: Vec<Item>) -> Self {
        let mut ports = BTreeMap::new();
        ports.insert(name.to_string(), items);
        Self { ports, ..Default::default() }
    }

    /// Adds items to another port.
    pub fn and(mut self, name: &str, items: Vec<Item>) -> Self {
        self.ports.entry(name.to_string()).or_default().extend(items);
        self
    }

    /// The node is waiting for approval `approval_id`.
    pub fn waiting(approval_id: String) -> Self {
        Self { waiting_for: Some(approval_id), ..Default::default() }
    }
}

/// A settings problem found by validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// JSON pointer to the setting, e.g. `/instructions`.
    pub path: String,
    pub message: String,
}

/// Errors from node execution.
#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error("node type `{0}` is not implemented yet")]
    NotImplemented(String),
    #[error("invalid settings: {0:?}")]
    InvalidSettings(Vec<ValidationIssue>),
    #[error("missing credential `{0}`")]
    MissingCredential(String),
    #[error("cancelled")]
    Cancelled,
    #[error("timed out")]
    Timeout,
    /// A transient failure the engine may retry.
    #[error("retryable: {0}")]
    Retryable(String),
    #[error("{0}")]
    Failed(String),
}

impl NodeError {
    /// Whether the engine should retry.
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Retryable(_) | Self::Timeout)
    }
}
