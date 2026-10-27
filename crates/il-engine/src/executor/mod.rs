//! Graph executor (EN-1, EN-2, EN-7): per-item/batch nodes, named branch outputs, merges,
//! parallel branches with a concurrency limit, retries with backoff, per-node timeouts and
//! cancellation. `Engine` implements `FlowRunner`.

mod driver;
mod invoke;
mod runner;

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use il_core::RunStatus;
use il_sdk::NodeExecutor;
use il_sdk::Services;
use serde::{Deserialize, Serialize};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use crate::caps::CapsConfig;
use crate::events::EventBus;

/// Engine settings (the `[engine]` and `[caps]` config sections).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Nodes running at the same time within one run.
    pub max_parallel: usize,
    /// Retries after the first attempt for retryable errors.
    pub retries: u32,
    /// First retry delay; doubles each attempt.
    pub retry_backoff_ms: u64,
    /// Per node call.
    pub node_timeout_ms: u64,
    pub caps: CapsConfig,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            retries: 2,
            retry_backoff_ms: 500,
            node_timeout_ms: 300_000,
            caps: CapsConfig::default(),
        }
    }
}

impl EngineConfig {
    fn node_timeout(&self) -> Duration {
        Duration::from_millis(self.node_timeout_ms)
    }
}

/// Engine errors (surfaced as `RunnerError` through `FlowRunner`).
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("flow not found: {0}")]
    FlowNotFound(String),
    #[error("run not found: {0}")]
    RunNotFound(String),
    #[error("approval not found or already decided: {0}")]
    Approval(String),
    #[error("invalid flow: {0}")]
    InvalidFlow(String),
    #[error("storage: {0}")]
    Store(String),
}

/// Finds node executors by type id. Defaults to the global inventory registry; tests and the
/// server may add or override types.
#[derive(Clone, Default)]
pub struct NodeLookup {
    extra: BTreeMap<String, Arc<dyn NodeExecutor>>,
}

impl NodeLookup {
    /// Adds (or overrides) a node type.
    pub fn with(mut self, node: Arc<dyn NodeExecutor>) -> Self {
        self.extra.insert(node.manifest().type_id.clone(), node);
        self
    }

    /// The executor for a type id.
    pub fn get(&self, type_id: &str) -> Option<Arc<dyn NodeExecutor>> {
        self.extra.get(type_id).cloned().or_else(|| il_sdk::registry::node(type_id))
    }
}

pub(crate) struct Inner {
    pub services: Services,
    pub nodes: NodeLookup,
    pub bus: EventBus,
    pub config: EngineConfig,
    pub cancels: Mutex<HashMap<String, CancellationToken>>,
    pub status: Mutex<HashMap<String, watch::Sender<RunStatus>>>,
}

/// The flow engine. Cheap to clone.
#[derive(Clone)]
pub struct Engine {
    pub(crate) inner: Arc<Inner>,
}

impl Engine {
    /// Creates an engine over the services.
    pub fn new(services: Services, nodes: NodeLookup, config: EngineConfig) -> Self {
        let bus = EventBus::new(services.store.clone(), services.clock.clone());
        Self {
            inner: Arc::new(Inner {
                services,
                nodes,
                bus,
                config,
                cancels: Mutex::new(HashMap::new()),
                status: Mutex::new(HashMap::new()),
            }),
        }
    }

    /// The live event bus (for SSE).
    pub fn events(&self) -> &EventBus {
        &self.inner.bus
    }

    /// Waits until the run finishes, pauses for a human, or hits a cap; returns the status.
    pub async fn wait(&self, run_id: &str) -> RunStatus {
        let known = self.inner.status.lock().map(|m| m.contains_key(run_id)).unwrap_or(false);
        if !known
            && let Some(s) = self.current_status(run_id).await
            && !matches!(s, RunStatus::Queued | RunStatus::Running)
        {
            return s;
        }
        let mut rx = self.inner.status_rx(run_id);
        loop {
            let status = *rx.borrow_and_update();
            if !matches!(status, RunStatus::Queued | RunStatus::Running) {
                return status;
            }
            if rx.changed().await.is_err() {
                return status;
            }
        }
    }
}

impl Inner {
    pub(crate) fn set_status(&self, run_id: &str, status: RunStatus) {
        if let Ok(mut map) = self.status.lock() {
            map.entry(run_id.to_string()).or_insert_with(|| watch::channel(status).0).send_replace(status);
        }
    }

    pub(crate) fn status_rx(&self, run_id: &str) -> watch::Receiver<RunStatus> {
        match self.status.lock() {
            Ok(mut map) => {
                map.entry(run_id.to_string()).or_insert_with(|| watch::channel(RunStatus::Queued).0).subscribe()
            }
            Err(_) => watch::channel(RunStatus::Failed).1,
        }
    }

    pub(crate) fn cancel_token(&self, run_id: &str) -> CancellationToken {
        match self.cancels.lock() {
            Ok(mut map) => map.entry(run_id.to_string()).or_default().clone(),
            Err(_) => CancellationToken::new(),
        }
    }
}
