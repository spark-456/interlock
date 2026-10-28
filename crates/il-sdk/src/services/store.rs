//! Persistence trait. `il-store` implements it on SQLite; `testkit::MemoryStore` in memory.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use il_core::{Approval, BuildEvent, Flow, NodeRun, Run, RunEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Store errors.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("storage: {0}")]
    Backend(String),
}

/// Result alias.
pub type StoreResult<T> = Result<T, StoreError>;

/// A decrypted credential, only ever handed to nodes through `NodeCtx::credential`.
#[derive(Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub app: String,
    /// Secret fields (`access_token`, `api_key`, `password`…). Never logged.
    pub secret: Value,
    /// Non-secret metadata (account name, scopes).
    pub meta: Value,
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential").field("id", &self.id).field("app", &self.app).finish_non_exhaustive()
    }
}

/// A stored credential without its secret, for listing connections.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CredentialInfo {
    pub id: String,
    pub app: String,
    pub meta: Value,
}

/// One cost entry (us_ledger).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpendEntry {
    pub workspace_id: String,
    pub flow_id: Option<String>,
    pub run_id: Option<String>,
    pub build_id: Option<String>,
    /// `architect`, `agent`, `fast`, `embed`, `media`…
    pub role: String,
    pub cost_usd: f64,
    pub at: DateTime<Utc>,
}

/// Which spend total to read.
#[derive(Debug, Clone, PartialEq)]
pub enum SpendScope {
    Run(String),
    FlowSince(String, DateTime<Utc>),
    WorkspaceSince(String, DateTime<Utc>),
    GlobalSince(DateTime<Utc>),
}

/// Persistence used by engine, architect, publish and server.
#[async_trait]
pub trait Store: Send + Sync {
    // Flows (every put creates a version).
    async fn get_flow(&self, id: &str) -> StoreResult<Option<Flow>>;
    async fn put_flow(&self, flow: &Flow) -> StoreResult<u32>;
    async fn list_flows(&self, workspace_id: &str) -> StoreResult<Vec<Flow>>;
    async fn delete_flow(&self, id: &str) -> StoreResult<()>;
    async fn flow_version(&self, id: &str, version: u32) -> StoreResult<Option<Flow>>;
    async fn set_flow_active(&self, id: &str, active: bool) -> StoreResult<()>;
    async fn is_flow_active(&self, id: &str) -> StoreResult<bool>;
    /// Active flows to restore triggers after restart.
    async fn active_flows(&self) -> StoreResult<Vec<Flow>>;

    // Runs.
    async fn put_run(&self, run: &Run) -> StoreResult<()>;
    async fn get_run(&self, id: &str) -> StoreResult<Option<Run>>;
    async fn list_runs(&self, flow_id: &str, limit: usize) -> StoreResult<Vec<Run>>;
    /// Nonterminal runs to recover after restart.
    async fn recoverable_runs(&self) -> StoreResult<Vec<Run>>;
    async fn put_node_run(&self, node_run: &NodeRun) -> StoreResult<()>;
    async fn node_runs(&self, run_id: &str) -> StoreResult<Vec<NodeRun>>;

    // Events (append-only, ordered by seq per stream).
    async fn append_run_event(&self, event: &RunEvent) -> StoreResult<()>;
    async fn run_events(&self, run_id: &str, after: u64) -> StoreResult<Vec<RunEvent>>;
    async fn append_build_event(&self, event: &BuildEvent) -> StoreResult<()>;
    async fn build_events(&self, build_id: &str, after: u64) -> StoreResult<Vec<BuildEvent>>;

    // Approvals.
    async fn put_approval(&self, approval: &Approval) -> StoreResult<()>;
    async fn get_approval(&self, id: &str) -> StoreResult<Option<Approval>>;
    async fn list_approvals(&self, workspace_id: &str, pending_only: bool) -> StoreResult<Vec<Approval>>;
    /// Pending approvals whose deadline has passed and whose policy takes a decision.
    async fn due_approvals(&self, now: DateTime<Utc>) -> StoreResult<Vec<Approval>>;
    /// Atomically changes a pending approval to its decided state.
    async fn transition_approval(&self, approval: &Approval) -> StoreResult<bool>;

    // Credentials (decrypted on read).
    async fn credential(&self, workspace_id: &str, id: &str) -> StoreResult<Option<Credential>>;
    /// Persists an encrypted credential or updates its token after refresh.
    async fn put_credential(&self, workspace_id: &str, credential: &Credential) -> StoreResult<()>;
    /// The workspace's credentials, without secrets.
    async fn credentials(&self, workspace_id: &str) -> StoreResult<Vec<CredentialInfo>>;

    // Small key/value state: poll cursors, trigger state, dedupe keys.
    async fn kv_get(&self, key: &str) -> StoreResult<Option<Value>>;
    async fn kv_put(&self, key: &str, value: &Value) -> StoreResult<()>;

    // Spend ledger.
    async fn record_spend(&self, entry: &SpendEntry) -> StoreResult<()>;
    async fn spend(&self, scope: &SpendScope) -> StoreResult<f64>;
    /// A workspace's spend entries since `since`, oldest first.
    async fn spend_entries(&self, workspace_id: &str, since: DateTime<Utc>) -> StoreResult<Vec<SpendEntry>>;
}
