//! Human pauses (EN-3): nodes call `ctx.request_approval`, the engine persists a pending
//! `Approval`, notifies the requested channels and pauses that item. `Engine::decide`
//! resumes it, after a restart too, because the run checkpoint lives in the store.

use std::sync::Arc;

use async_trait::async_trait;
use il_core::{Approval, ApprovalStatus};
use il_sdk::NodeError;
use il_sdk::ctx::{ApprovalGate, ApprovalSpec, RunInfo};
use il_sdk::services::{Clock, Notification, Notifier, Store};

/// The engine's `ApprovalGate`.
pub struct EngineGate {
    pub store: Arc<dyn Store>,
    pub notifier: Arc<dyn Notifier>,
    pub clock: Arc<dyn Clock>,
}

#[async_trait]
impl ApprovalGate for EngineGate {
    async fn request(&self, run: &RunInfo, spec: ApprovalSpec) -> Result<String, NodeError> {
        let id = il_sdk::id("apr");
        let channels = if spec.channels.is_empty() { vec!["inapp".to_string()] } else { spec.channels.clone() };
        let expires_at = match spec.timeout_hours {
            Some(hours) if hours.is_finite() && hours > 0.0 && hours <= 876_000.0 => {
                Some(self.clock.now() + chrono::Duration::milliseconds((hours * 3_600_000.0).round() as i64))
            }
            Some(_) => return Err(NodeError::Failed("invalid approval timeout".into())),
            None => None,
        };
        let approval = Approval {
            id: id.clone(),
            workspace_id: run.workspace_id.clone(),
            flow_id: run.flow_id.clone(),
            run_id: run.run_id.clone(),
            node_id: run.node_id.clone(),
            title: spec.title.clone(),
            message: spec.message.clone(),
            payload: spec.payload.clone(),
            status: ApprovalStatus::Pending,
            channels: channels.clone(),
            created_at: self.clock.now(),
            expires_at,
            on_timeout: Some(spec.on_timeout),
            decided_at: None,
            decided_by: None,
        };
        self.store.put_approval(&approval).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        for channel in channels.iter().filter(|c| c.as_str() != "inapp") {
            let note = Notification {
                workspace_id: run.workspace_id.clone(),
                channel: channel.clone(),
                target: spec.target.clone(),
                title: spec.title.clone(),
                body: spec.message.clone(),
                approval_id: Some(id.clone()),
                payload: spec.payload.clone(),
                credential_id: None,
            };
            if let Err(e) = self.notifier.send(&note).await {
                tracing::warn!(approval_id = %id, channel = %channel, error = %e, "approval notification failed");
            }
        }
        Ok(id)
    }
}
