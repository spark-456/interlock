//! `NodeCtx`: the only way a node touches the outside world (AGENTS.md §5.7).

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use il_core::ApprovalDecision;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use crate::http::SafeHttp;
use crate::node::NodeError;
use crate::services::{Credential, FileStore, FlowRunner, KnowledgeService, MediaGen, ModelGateway, Services};

/// Identity of the current node execution.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RunInfo {
    pub run_id: String,
    pub flow_id: String,
    pub workspace_id: String,
    pub node_id: String,
    pub node_name: String,
    pub trigger: String,
    pub attempt: u32,
    /// Index of the item for per-item execution.
    pub item_index: usize,
    /// True for test runs and demo replays (media providers default to `recorded`).
    pub test: bool,
}

/// What a node asks a human to decide.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApprovalSpec {
    pub title: String,
    pub message: String,
    pub payload: Value,
    /// `inapp`, `slack`…
    pub channels: Vec<String>,
    /// Channel destination from settings (e.g. Slack channel).
    pub target: Option<String>,
    pub timeout_hours: Option<f64>,
}

/// Receives progress from nodes (the engine forwards it as `node.progress`).
pub trait ProgressSink: Send + Sync {
    /// Reports progress for the current node.
    fn progress(&self, run: &RunInfo, pct: f32, message: &str);
}

/// Creates approvals (the engine persists them and pauses the run).
#[async_trait]
pub trait ApprovalGate: Send + Sync {
    /// Creates a pending approval and returns its id.
    async fn request(&self, run: &RunInfo, spec: ApprovalSpec) -> Result<String, NodeError>;
}

/// Context passed to `NodeExecutor::execute`.
pub struct NodeCtx {
    /// Settings with expressions already rendered for this item.
    pub settings: Value,
    pub run: RunInfo,
    pub services: Services,
    /// Node's credential bindings: credential key → credential id.
    pub credential_ids: BTreeMap<String, String>,
    pub cancel: CancellationToken,
    pub progress: Arc<dyn ProgressSink>,
    pub approvals: Arc<dyn ApprovalGate>,
    /// Set when the engine resumes a node after a human decision.
    pub resume: Option<ApprovalDecision>,
}

impl NodeCtx {
    /// Looks up and decrypts the credential bound to `key`.
    pub async fn credential(&self, key: &str) -> Result<Credential, NodeError> {
        let id = self.credential_ids.get(key).ok_or_else(|| NodeError::MissingCredential(key.into()))?;
        self.services
            .store
            .credential(&self.run.workspace_id, id)
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .ok_or_else(|| NodeError::MissingCredential(key.into()))
    }

    /// Model gateway.
    pub fn models(&self) -> &Arc<dyn ModelGateway> {
        &self.services.models
    }
    /// File store.
    pub fn files(&self) -> &Arc<dyn FileStore> {
        &self.services.files
    }
    /// Knowledge service.
    pub fn knowledge(&self) -> &Arc<dyn KnowledgeService> {
        &self.services.knowledge
    }
    /// Media generation.
    pub fn media(&self) -> &Arc<dyn MediaGen> {
        &self.services.media
    }
    /// SSRF-safe HTTP client.
    pub fn http(&self) -> &SafeHttp {
        &self.services.http
    }
    /// Reports progress (0–100).
    pub fn emit_progress(&self, pct: f32, message: &str) {
        self.progress.progress(&self.run, pct, message);
    }
    /// Creates an approval; return `NodeOutput::waiting(id)` afterwards.
    pub async fn request_approval(&self, spec: ApprovalSpec) -> Result<String, NodeError> {
        self.approvals.request(&self.run, spec).await
    }
    /// Cancellation token for the run.
    pub fn cancel_token(&self) -> &CancellationToken {
        &self.cancel
    }
    /// Current run identity.
    pub fn run_info(&self) -> &RunInfo {
        &self.run
    }
    /// Stable key for idempotent external sends: one per node run and item.
    pub fn dedupe_key(&self) -> String {
        format!("{}:{}:{}", self.run.run_id, self.run.node_id, self.run.item_index)
    }
    /// Typed setting helper.
    pub fn setting<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.settings.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

/// Context passed to `TriggerNode::start`.
pub struct TriggerCtx {
    pub flow_id: String,
    pub workspace_id: String,
    pub node_id: String,
    pub settings: Value,
    pub services: Services,
    pub runner: Arc<dyn FlowRunner>,
    pub cancel: CancellationToken,
}

/// Stops a running trigger when dropped or on `stop()`.
pub struct TriggerHandle {
    cancel: CancellationToken,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl TriggerHandle {
    /// Wraps a background task.
    pub fn new(cancel: CancellationToken, task: Option<tokio::task::JoinHandle<()>>) -> Self {
        Self { cancel, task }
    }
    /// A handle for triggers with no background work (manual, webhook bind).
    pub fn noop() -> Self {
        Self { cancel: CancellationToken::new(), task: None }
    }
    /// Stops the trigger.
    pub async fn stop(mut self) {
        self.cancel.cancel();
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

impl Drop for TriggerHandle {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
