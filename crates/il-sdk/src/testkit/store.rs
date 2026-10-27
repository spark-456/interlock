//! In-memory `Store`.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Mutex;

use async_trait::async_trait;
use il_core::{Approval, ApprovalStatus, ApprovalTimeout, BuildEvent, Flow, NodeRun, Run, RunEvent};
use serde_json::Value;

use crate::services::store::{Credential, SpendEntry, SpendScope, Store, StoreError, StoreResult};

#[derive(Default)]
struct Inner {
    flows: HashMap<String, Vec<Flow>>,
    active: HashSet<String>,
    runs: HashMap<String, Run>,
    node_runs: HashMap<String, BTreeMap<String, NodeRun>>,
    run_events: HashMap<String, Vec<RunEvent>>,
    build_events: HashMap<String, Vec<BuildEvent>>,
    approvals: HashMap<String, Approval>,
    credentials: HashMap<String, Credential>,
    kv: HashMap<String, Value>,
    spend: Vec<SpendEntry>,
}

/// Everything in a mutex; fine for tests and fake mode.
#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

impl MemoryStore {
    fn lock(&self) -> StoreResult<std::sync::MutexGuard<'_, Inner>> {
        self.inner.lock().map_err(|_| StoreError::Backend("poisoned".into()))
    }

    /// Seeds a credential (tests).
    pub fn add_credential(&self, cred: Credential) {
        if let Ok(mut g) = self.inner.lock() {
            g.credentials.insert(cred.id.clone(), cred);
        }
    }
}

#[async_trait]
impl Store for MemoryStore {
    async fn get_flow(&self, id: &str) -> StoreResult<Option<Flow>> {
        Ok(self.lock()?.flows.get(id).and_then(|v| v.last().cloned()))
    }
    async fn put_flow(&self, flow: &Flow) -> StoreResult<u32> {
        let mut g = self.lock()?;
        let versions = g.flows.entry(flow.id.clone()).or_default();
        let mut f = flow.clone();
        f.version = versions.last().map(|v| v.version + 1).unwrap_or(1);
        let v = f.version;
        versions.push(f);
        Ok(v)
    }
    async fn list_flows(&self, workspace_id: &str) -> StoreResult<Vec<Flow>> {
        let g = self.lock()?;
        let mut out: Vec<Flow> =
            g.flows.values().filter_map(|v| v.last()).filter(|f| f.workspace_id == workspace_id).cloned().collect();
        out.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(out)
    }
    async fn delete_flow(&self, id: &str) -> StoreResult<()> {
        self.lock()?.flows.remove(id);
        Ok(())
    }
    async fn flow_version(&self, id: &str, version: u32) -> StoreResult<Option<Flow>> {
        Ok(self.lock()?.flows.get(id).and_then(|v| v.iter().find(|f| f.version == version).cloned()))
    }
    async fn set_flow_active(&self, id: &str, active: bool) -> StoreResult<()> {
        let mut g = self.lock()?;
        if active {
            g.active.insert(id.into());
        } else {
            g.active.remove(id);
        }
        Ok(())
    }
    async fn is_flow_active(&self, id: &str) -> StoreResult<bool> {
        Ok(self.lock()?.active.contains(id))
    }
    async fn active_flows(&self) -> StoreResult<Vec<Flow>> {
        let g = self.lock()?;
        let mut flows: Vec<_> = g
            .flows
            .values()
            .filter_map(|versions| versions.last())
            .filter(|flow| g.active.contains(&flow.id))
            .cloned()
            .collect();
        flows.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(flows)
    }
    async fn put_run(&self, run: &Run) -> StoreResult<()> {
        self.lock()?.runs.insert(run.id.clone(), run.clone());
        Ok(())
    }
    async fn get_run(&self, id: &str) -> StoreResult<Option<Run>> {
        Ok(self.lock()?.runs.get(id).cloned())
    }
    async fn list_runs(&self, flow_id: &str, limit: usize) -> StoreResult<Vec<Run>> {
        let g = self.lock()?;
        let mut out: Vec<Run> = g.runs.values().filter(|r| r.flow_id == flow_id).cloned().collect();
        out.sort_by_key(|r| std::cmp::Reverse(r.started_at));
        out.truncate(limit);
        Ok(out)
    }
    async fn recoverable_runs(&self) -> StoreResult<Vec<Run>> {
        let g = self.lock()?;
        let mut runs: Vec<_> = g
            .runs
            .values()
            .filter(|run| {
                matches!(
                    run.status,
                    il_core::RunStatus::Queued | il_core::RunStatus::Running | il_core::RunStatus::Waiting
                )
            })
            .cloned()
            .collect();
        runs.sort_by_key(|run| run.started_at);
        Ok(runs)
    }
    async fn put_node_run(&self, nr: &NodeRun) -> StoreResult<()> {
        self.lock()?.node_runs.entry(nr.run_id.clone()).or_default().insert(nr.node_id.clone(), nr.clone());
        Ok(())
    }
    async fn node_runs(&self, run_id: &str) -> StoreResult<Vec<NodeRun>> {
        Ok(self.lock()?.node_runs.get(run_id).map(|m| m.values().cloned().collect()).unwrap_or_default())
    }
    async fn append_run_event(&self, ev: &RunEvent) -> StoreResult<()> {
        let mut g = self.lock()?;
        let list = g.run_events.entry(ev.run_id.clone()).or_default();
        if list.last().is_some_and(|l| l.seq >= ev.seq) {
            return Err(StoreError::Conflict(format!("seq {} out of order", ev.seq)));
        }
        list.push(ev.clone());
        Ok(())
    }
    async fn run_events(&self, run_id: &str, after: u64) -> StoreResult<Vec<RunEvent>> {
        Ok(self
            .lock()?
            .run_events
            .get(run_id)
            .map(|l| l.iter().filter(|e| e.seq > after).cloned().collect())
            .unwrap_or_default())
    }
    async fn append_build_event(&self, ev: &BuildEvent) -> StoreResult<()> {
        let mut g = self.lock()?;
        let list = g.build_events.entry(ev.build_id.clone()).or_default();
        if list.last().is_some_and(|l| l.seq >= ev.seq) {
            return Err(StoreError::Conflict(format!("seq {} out of order", ev.seq)));
        }
        list.push(ev.clone());
        Ok(())
    }
    async fn build_events(&self, build_id: &str, after: u64) -> StoreResult<Vec<BuildEvent>> {
        Ok(self
            .lock()?
            .build_events
            .get(build_id)
            .map(|l| l.iter().filter(|e| e.seq > after).cloned().collect())
            .unwrap_or_default())
    }
    async fn put_approval(&self, a: &Approval) -> StoreResult<()> {
        self.lock()?.approvals.insert(a.id.clone(), a.clone());
        Ok(())
    }
    async fn get_approval(&self, id: &str) -> StoreResult<Option<Approval>> {
        Ok(self.lock()?.approvals.get(id).cloned())
    }
    async fn list_approvals(&self, workspace_id: &str, pending_only: bool) -> StoreResult<Vec<Approval>> {
        let g = self.lock()?;
        let mut out: Vec<Approval> = g
            .approvals
            .values()
            .filter(|a| a.workspace_id == workspace_id && (!pending_only || a.status == ApprovalStatus::Pending))
            .cloned()
            .collect();
        out.sort_by_key(|a| std::cmp::Reverse(a.created_at));
        Ok(out)
    }
    async fn due_approvals(&self, now: chrono::DateTime<chrono::Utc>) -> StoreResult<Vec<Approval>> {
        Ok(self
            .lock()?
            .approvals
            .values()
            .filter(|approval| {
                approval.status == ApprovalStatus::Pending
                    && approval.expires_at.is_some_and(|deadline| deadline <= now)
                    && !matches!(approval.on_timeout, None | Some(ApprovalTimeout::KeepWaiting))
            })
            .cloned()
            .collect())
    }
    async fn transition_approval(&self, approval: &Approval) -> StoreResult<bool> {
        let mut inner = self.lock()?;
        let Some(current) = inner.approvals.get(&approval.id) else { return Ok(false) };
        if current.status != ApprovalStatus::Pending {
            return Ok(false);
        }
        inner.approvals.insert(approval.id.clone(), approval.clone());
        Ok(true)
    }
    async fn credential(&self, _workspace_id: &str, id: &str) -> StoreResult<Option<Credential>> {
        Ok(self.lock()?.credentials.get(id).cloned())
    }
    async fn put_credential(&self, _workspace_id: &str, credential: &Credential) -> StoreResult<()> {
        self.lock()?.credentials.insert(credential.id.clone(), credential.clone());
        Ok(())
    }
    async fn kv_get(&self, key: &str) -> StoreResult<Option<Value>> {
        Ok(self.lock()?.kv.get(key).cloned())
    }
    async fn kv_put(&self, key: &str, value: &Value) -> StoreResult<()> {
        self.lock()?.kv.insert(key.into(), value.clone());
        Ok(())
    }
    async fn record_spend(&self, entry: &SpendEntry) -> StoreResult<()> {
        self.lock()?.spend.push(entry.clone());
        Ok(())
    }
    async fn spend(&self, scope: &SpendScope) -> StoreResult<f64> {
        let g = self.lock()?;
        let total = g
            .spend
            .iter()
            .filter(|e| match scope {
                SpendScope::Run(id) => e.run_id.as_deref() == Some(id),
                SpendScope::FlowSince(id, t) => e.flow_id.as_deref() == Some(id) && e.at >= *t,
                SpendScope::WorkspaceSince(id, t) => &e.workspace_id == id && e.at >= *t,
                SpendScope::GlobalSince(t) => e.at >= *t,
            })
            .map(|e| e.cost_usd)
            .sum();
        Ok(total)
    }
}
