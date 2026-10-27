//! `FlowRunner` for `Engine`: start, invoke, decide, cancel, replay; plus restart recovery.

use async_trait::async_trait;
use il_core::{ApprovalDecision, ApprovalStatus, ApprovalTimeout, Decision, Flow, Item, Run, RunEventKind, RunStatus};
use il_sdk::services::runner::InvokeOutcome;
use il_sdk::services::{FlowRunner, RunnerError, StartRun};

use super::driver::drive;
use super::invoke::Resume;
use super::{Engine, EngineError};
use crate::state::{Checkpoint, Ports};

impl From<EngineError> for RunnerError {
    fn from(e: EngineError) -> Self {
        match e {
            EngineError::FlowNotFound(id) => RunnerError::FlowNotFound(id),
            EngineError::RunNotFound(id) => RunnerError::RunNotFound(id),
            EngineError::InvalidFlow(m) => RunnerError::InvalidFlow(m),
            other => RunnerError::Other(other.to_string()),
        }
    }
}

impl Engine {
    async fn load_flow(&self, flow_id: &str, version: Option<u32>) -> Result<Flow, EngineError> {
        let store = &self.inner.services.store;
        let found = match version {
            Some(v) => store.flow_version(flow_id, v).await,
            None => store.get_flow(flow_id).await,
        };
        match found.map_err(|e| EngineError::Store(e.to_string()))? {
            Some(f) => Ok(f),
            None if version.is_some() => {
                store.get_flow(flow_id).await.ok().flatten().ok_or_else(|| EngineError::FlowNotFound(flow_id.into()))
            }
            None => Err(EngineError::FlowNotFound(flow_id.into())),
        }
    }

    /// The trigger node a run starts from: the requested one, else the first trigger, else the first root.
    fn trigger_node(&self, flow: &Flow, requested: Option<&str>) -> Result<String, EngineError> {
        if let Some(id) = requested {
            return flow
                .node(id)
                .map(|n| n.id.clone())
                .ok_or_else(|| EngineError::InvalidFlow(format!("no node `{id}`")));
        }
        let is_trigger = |t: &str| self.inner.nodes.get(t).is_some_and(|n| n.manifest().trigger);
        flow.nodes
            .iter()
            .find(|n| is_trigger(&n.node_type))
            .or_else(|| flow.roots().next())
            .map(|n| n.id.clone())
            .ok_or_else(|| EngineError::InvalidFlow("the flow has no starting node".into()))
    }

    /// Starts a run and returns it without waiting.
    async fn start_run(&self, req: StartRun) -> Result<String, EngineError> {
        let flow = self.load_flow(&req.flow_id, req.version).await?;
        let trigger_node = self.trigger_node(&flow, req.trigger_node.as_deref())?;
        let inner = &self.inner;
        let run = Run {
            id: il_sdk::id("run"),
            flow_id: flow.id.clone(),
            workspace_id: flow.workspace_id.clone(),
            flow_version: flow.version,
            status: RunStatus::Queued,
            trigger: req.trigger.clone(),
            started_at: inner.services.clock.now(),
            finished_at: None,
            cost_usd: 0.0,
            error: None,
            replay: false,
            state: serde_json::Value::Null,
        };
        inner.set_status(&run.id, RunStatus::Queued);
        inner.services.store.put_run(&run).await.map_err(|e| EngineError::Store(e.to_string()))?;
        let kind = RunEventKind::RunStarted { flow_id: flow.id.clone(), trigger: req.trigger.clone() };
        inner.bus.emit(&run.id, None, kind).await;

        // The trigger node's output is the request's items.
        let items: Vec<Item> = req.items;
        inner.bus.emit(&run.id, Some(&trigger_node), RunEventKind::NodeStarted {}).await;
        let preview = items.iter().take(3).cloned().collect();
        let kind = RunEventKind::NodeItems { port: "out".into(), items: preview, total: items.len() as u64 };
        inner.bus.emit(&run.id, Some(&trigger_node), kind).await;
        let kind = RunEventKind::NodeSucceeded { count: items.len() as u64, ms: 0, cost_usd: 0.0 };
        inner.bus.emit(&run.id, Some(&trigger_node), kind).await;
        let mut ports = Ports::new();
        ports.insert("out".into(), items);
        let mut cp = Checkpoint { trigger_node: trigger_node.clone(), ..Default::default() };
        cp.outputs.insert(trigger_node, ports);

        let id = run.id.clone();
        tokio::spawn(drive(inner.clone(), run, flow, cp, None));
        Ok(id)
    }

    /// Applies a decision and resumes the paused node.
    async fn decide_approval(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        by: Option<String>,
    ) -> Result<(), EngineError> {
        self.decide_approval_internal(approval_id, decision, by, false).await
    }

    /// Applies due approval policies, including after a server restart.
    pub async fn expire_due_approvals(&self) -> Result<usize, EngineError> {
        let due = self
            .inner
            .services
            .store
            .due_approvals(self.inner.services.clock.now())
            .await
            .map_err(|e| EngineError::Store(e.to_string()))?;
        let mut completed = 0;
        for approval in due {
            let decision = match approval.on_timeout {
                Some(ApprovalTimeout::Approve) => Decision::Approve,
                Some(ApprovalTimeout::Reject) => Decision::Reject,
                _ => continue,
            };
            let outcome = self
                .decide_approval_internal(
                    &approval.id,
                    ApprovalDecision { decision, payload: None, comment: None },
                    Some("timeout".into()),
                    true,
                )
                .await;
            match outcome {
                Ok(()) => completed += 1,
                Err(EngineError::Approval(_)) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(completed)
    }

    async fn decide_approval_internal(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        by: Option<String>,
        timeout: bool,
    ) -> Result<(), EngineError> {
        let inner = &self.inner;
        let store = &inner.services.store;
        let mut approval = store
            .get_approval(approval_id)
            .await
            .map_err(|e| EngineError::Store(e.to_string()))?
            .filter(|a| a.status == ApprovalStatus::Pending)
            .ok_or_else(|| EngineError::Approval(approval_id.into()))?;
        let overdue = approval.expires_at.is_some_and(|deadline| deadline <= inner.services.clock.now());
        let policy = approval.on_timeout.unwrap_or(ApprovalTimeout::KeepWaiting);
        let matches_policy = matches!(
            (policy, decision.decision),
            (ApprovalTimeout::Approve, Decision::Approve) | (ApprovalTimeout::Reject, Decision::Reject)
        );
        if (timeout && (!overdue || !matches_policy)) || (!timeout && overdue && policy != ApprovalTimeout::KeepWaiting)
        {
            return Err(EngineError::Approval(approval_id.into()));
        }
        approval.status = match decision.decision {
            Decision::Approve => ApprovalStatus::Approved,
            Decision::Reject => ApprovalStatus::Rejected,
        };
        approval.decided_at = Some(inner.services.clock.now());
        approval.decided_by = by.clone();
        if let Some(p) = &decision.payload {
            approval.payload = p.clone();
        }
        if !store.transition_approval(&approval).await.map_err(|e| EngineError::Store(e.to_string()))? {
            return Err(EngineError::Approval(approval_id.into()));
        }
        let kind = RunEventKind::ApprovalDecided { approval_id: approval_id.into(), decision: decision.decision, by };
        inner.bus.emit(&approval.run_id, Some(&approval.node_id), kind).await;

        let run = store
            .get_run(&approval.run_id)
            .await
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or_else(|| EngineError::RunNotFound(approval.run_id.clone()))?;
        let flow = self.load_flow(&run.flow_id, Some(run.flow_version)).await?;
        let cp = Checkpoint::from_state(&run.state);
        let partial = cp
            .partial
            .get(&approval.node_id)
            .filter(|p| p.waiting.contains_key(approval_id))
            .cloned()
            .ok_or_else(|| EngineError::Approval(approval_id.into()))?;
        let resume = Resume { partial, approval_id: approval_id.into(), decision };
        inner.set_status(&run.id, RunStatus::Running);
        tokio::spawn(drive(inner.clone(), run, flow, cp, Some((approval.node_id.clone(), resume))));
        Ok(())
    }

    /// Continues a run that was `Running` when the server stopped (call at startup).
    pub async fn recover(&self, run_id: &str) -> Result<(), EngineError> {
        let run = self
            .inner
            .services
            .store
            .get_run(run_id)
            .await
            .map_err(|e| EngineError::Store(e.to_string()))?
            .ok_or_else(|| EngineError::RunNotFound(run_id.into()))?;
        if run.status != RunStatus::Running && run.status != RunStatus::Queued {
            return Ok(());
        }
        let flow = self.load_flow(&run.flow_id, Some(run.flow_version)).await?;
        let cp = Checkpoint::from_state(&run.state);
        self.inner.set_status(&run.id, RunStatus::Running);
        tokio::spawn(drive(self.inner.clone(), run, flow, cp, None));
        Ok(())
    }

    /// Status from memory, or the store for runs started by another process.
    pub(crate) async fn current_status(&self, run_id: &str) -> Option<RunStatus> {
        self.inner.services.store.get_run(run_id).await.ok().flatten().map(|r| r.status)
    }
}

#[async_trait]
impl FlowRunner for Engine {
    async fn start(&self, req: StartRun) -> Result<String, RunnerError> {
        Ok(self.start_run(req).await?)
    }

    async fn invoke(&self, req: StartRun) -> Result<InvokeOutcome, RunnerError> {
        let id = self.start_run(req).await?;
        self.wait(&id).await;
        let run = self
            .inner
            .services
            .store
            .get_run(&id)
            .await
            .map_err(|e| RunnerError::Other(e.to_string()))?
            .ok_or_else(|| RunnerError::RunNotFound(id.clone()))?;
        let response = Checkpoint::from_state(&run.state).response;
        Ok(InvokeOutcome { run, response })
    }

    async fn decide(
        &self,
        approval_id: &str,
        decision: ApprovalDecision,
        by: Option<String>,
    ) -> Result<(), RunnerError> {
        Ok(self.decide_approval(approval_id, decision, by).await?)
    }

    async fn cancel(&self, run_id: &str) -> Result<(), RunnerError> {
        let token = self.inner.cancels.lock().ok().and_then(|m| m.get(run_id).cloned());
        match self.current_status(run_id).await {
            None => return Err(RunnerError::RunNotFound(run_id.into())),
            Some(RunStatus::Waiting | RunStatus::CapReached) => {
                // No driver is running: finish the run here.
                if let Ok(Some(mut run)) = self.inner.services.store.get_run(run_id).await {
                    run.status = RunStatus::Cancelled;
                    run.finished_at = Some(self.inner.services.clock.now());
                    let _ = self.inner.services.store.put_run(&run).await;
                    self.inner.bus.emit(run_id, None, RunEventKind::RunCancelled {}).await;
                    self.inner.set_status(run_id, RunStatus::Cancelled);
                }
            }
            Some(_) => {
                if let Some(t) = token {
                    t.cancel();
                }
            }
        }
        Ok(())
    }

    async fn replay(&self, recorded_run_id: &str) -> Result<String, RunnerError> {
        Ok(crate::replay::replay(self, recorded_run_id).await?)
    }
}
