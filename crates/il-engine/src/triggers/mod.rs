//! Trigger supervisor (EN-5): starts every trigger node of a flow when it is activated and stops
//! them on deactivate. Schedules, webhooks and polling live in the trigger nodes themselves
//! (`TriggerNode::start`); this module only manages their lifetimes. Chat, API and MCP calls
//! enter through [`TriggerSupervisor::invoke`], which returns the Respond output.

use std::collections::HashMap;
use std::sync::Arc;

use il_core::{Flow, Item};
use il_sdk::services::runner::InvokeOutcome;
use il_sdk::services::{FlowRunner, Services, StartRun};
use il_sdk::{TriggerCtx, TriggerHandle};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::executor::NodeLookup;

/// Trigger errors.
#[derive(Debug, thiserror::Error)]
pub enum TriggerError {
    #[error("flow not found: {0}")]
    FlowNotFound(String),
    #[error("the flow has no `{0}` trigger")]
    NoTrigger(String),
    #[error("trigger `{node}` failed to start: {message}")]
    Start { node: String, message: String },
    #[error("{0}")]
    Other(String),
}

/// Starts and stops flow triggers.
pub struct TriggerSupervisor {
    services: Services,
    runner: Arc<dyn FlowRunner>,
    nodes: NodeLookup,
    active: Mutex<HashMap<String, Vec<TriggerHandle>>>,
}

impl TriggerSupervisor {
    /// Creates a supervisor.
    pub fn new(services: Services, runner: Arc<dyn FlowRunner>, nodes: NodeLookup) -> Self {
        Self { services, runner, nodes, active: Mutex::new(HashMap::new()) }
    }

    async fn flow(&self, flow_id: &str) -> Result<Flow, TriggerError> {
        self.services
            .store
            .get_flow(flow_id)
            .await
            .map_err(|e| TriggerError::Other(e.to_string()))?
            .ok_or_else(|| TriggerError::FlowNotFound(flow_id.into()))
    }

    /// Starts every enabled trigger node of the flow (restarting any already running) and marks
    /// the flow active. Returns how many triggers were started. All-or-nothing.
    pub async fn activate(&self, flow_id: &str) -> Result<usize, TriggerError> {
        let flow = self.flow(flow_id).await?;
        self.stop(flow_id).await;
        let mut handles = Vec::new();
        for node in flow.nodes.iter().filter(|n| !n.disabled) {
            let Some(exec) = self.nodes.get(&node.node_type) else { continue };
            let Some(trigger) = exec.as_trigger() else { continue };
            let ctx = TriggerCtx {
                flow_id: flow.id.clone(),
                workspace_id: flow.workspace_id.clone(),
                node_id: node.id.clone(),
                settings: node.settings.clone(),
                credential_ids: node.credentials.clone(),
                services: self.services.clone(),
                runner: self.runner.clone(),
                cancel: CancellationToken::new(),
            };
            match trigger.start(ctx).await {
                Ok(h) => handles.push(h),
                Err(e) => {
                    for h in handles {
                        h.stop().await;
                    }
                    return Err(TriggerError::Start { node: node.id.clone(), message: e.to_string() });
                }
            }
        }
        let n = handles.len();
        self.active.lock().await.insert(flow_id.to_string(), handles);
        self.services.store.set_flow_active(flow_id, true).await.map_err(|e| TriggerError::Other(e.to_string()))?;
        tracing::info!(flow_id, triggers = n, "flow activated");
        Ok(n)
    }

    /// Stops the flow's triggers and marks it inactive.
    pub async fn deactivate(&self, flow_id: &str) -> Result<(), TriggerError> {
        self.stop(flow_id).await;
        self.services.store.set_flow_active(flow_id, false).await.map_err(|e| TriggerError::Other(e.to_string()))?;
        tracing::info!(flow_id, "flow deactivated");
        Ok(())
    }

    async fn stop(&self, flow_id: &str) {
        let handles = self.active.lock().await.remove(flow_id);
        for h in handles.into_iter().flatten() {
            h.stop().await;
        }
    }

    /// Whether triggers are running for the flow in this process.
    pub async fn is_running(&self, flow_id: &str) -> bool {
        self.active.lock().await.contains_key(flow_id)
    }

    /// Restarts triggers for flows that are marked active in the store (call at startup).
    pub async fn restore(&self, flow_ids: &[String]) -> usize {
        let mut n = 0;
        for id in flow_ids {
            if self.services.store.is_flow_active(id).await.unwrap_or(false) && self.activate(id).await.is_ok() {
                n += 1;
            }
        }
        n
    }

    /// Chat / API / MCP entry point: runs the flow from its `trigger_type` node and waits for
    /// the Respond output.
    pub async fn invoke(
        &self,
        flow_id: &str,
        trigger_type: &str,
        items: Vec<Item>,
    ) -> Result<InvokeOutcome, TriggerError> {
        let flow = self.flow(flow_id).await?;
        let node = flow
            .nodes
            .iter()
            .find(|n| n.node_type == trigger_type && !n.disabled)
            .ok_or_else(|| TriggerError::NoTrigger(trigger_type.into()))?;
        let req = StartRun {
            flow_id: flow.id.clone(),
            version: None,
            trigger: trigger_type.into(),
            items,
            trigger_node: Some(node.id.clone()),
            only: None,
        };
        self.runner.invoke(req).await.map_err(|e| TriggerError::Other(e.to_string()))
    }
}

#[async_trait::async_trait]
impl il_sdk::services::FlowActivation for TriggerSupervisor {
    async fn activate(&self, flow_id: &str) -> Result<usize, il_sdk::services::RunnerError> {
        TriggerSupervisor::activate(self, flow_id)
            .await
            .map_err(|e| il_sdk::services::RunnerError::Other(e.to_string()))
    }
    async fn deactivate(&self, flow_id: &str) -> Result<(), il_sdk::services::RunnerError> {
        TriggerSupervisor::deactivate(self, flow_id)
            .await
            .map_err(|e| il_sdk::services::RunnerError::Other(e.to_string()))
    }
}
