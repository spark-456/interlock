//! Calling one node: settings rendering per item, per-item vs batch, retries with backoff,
//! timeouts, cancellation and approval pauses.

use std::sync::Arc;
use std::time::{Duration, Instant};

use il_core::expr::{Scope, render_value};
use il_core::{ApprovalDecision, ExecMode, Item, Node, RunEventKind};
use il_sdk::ctx::{NodeCtx, ProgressSink, RunInfo};
use il_sdk::{NodeError, NodeExecutor, NodeInput, NodeOutput, Services};
use serde_json::{Value, json};
use tokio_util::sync::CancellationToken;

use super::Inner;
use crate::approvals::EngineGate;
use crate::state::{Partial, Ports};

/// What happened to a node call.
pub(crate) enum CallResult {
    Done { ports: Ports, cost_usd: f64, ms: u64, input: Vec<Item> },
    Waiting(Partial),
    Failed { error: String, retryable: bool, input: Vec<Item> },
}

/// A resumed item: the paused node's state, the approval being answered, and the decision.
pub(crate) struct Resume {
    pub partial: Partial,
    pub approval_id: String,
    pub decision: ApprovalDecision,
}

/// Runs one node over its input (all items, or the resumed item).
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_node(
    inner: Arc<Inner>,
    services: Services,
    base: RunInfo,
    node: Node,
    input: Ports,
    nodes_scope: Value,
    cancel: CancellationToken,
    resume: Option<Resume>,
) -> CallResult {
    let started = Instant::now();
    let main: Vec<Item> = input.get("in").cloned().unwrap_or_default();
    let Some(exec) = inner.nodes.get(&node.node_type) else {
        let error = format!("unknown node type `{}`", node.node_type);
        return CallResult::Failed { error, retryable: false, input: main };
    };
    if node.disabled {
        let mut ports = Ports::new();
        ports.insert("out".into(), main.clone());
        return CallResult::Done { ports, cost_usd: 0.0, ms: 0, input: main };
    }
    if let Err(issues) = exec.validate(&node.settings) {
        let error = format!(
            "invalid settings: {}",
            issues.iter().map(|i| format!("{} {}", i.path, i.message)).collect::<Vec<_>>().join("; ")
        );
        return CallResult::Failed { error, retryable: false, input: main };
    }

    let per_item = exec.manifest().exec_mode == ExecMode::PerItem && !main.is_empty();
    let (mut partial, calls): (Partial, Vec<(usize, Option<ApprovalDecision>)>) = match resume {
        Some(r) => {
            let mut p = r.partial;
            let idx = p.waiting.remove(&r.approval_id).unwrap_or(0);
            (p, vec![(idx, Some(r.decision))])
        }
        None => {
            let started_ms = inner.services.clock.now().timestamp_millis();
            let p = Partial { input: input.clone(), batch: !per_item, started_ms, ..Default::default() };
            let n = if per_item { main.len() } else { 1 };
            (p, (0..n).map(|i| (i, None)).collect())
        }
    };

    let all_json: Vec<Value> = main.iter().map(|i| i.json.clone()).collect();
    for (idx, decision) in calls {
        let node_input = if partial.batch {
            NodeInput { ports: partial.input.clone() }
        } else {
            NodeInput::items(main.get(idx).cloned().into_iter().collect())
        };
        let current = node_input.main().first().map(|i| i.json.clone()).unwrap_or(Value::Null);
        let scope = Scope::new()
            .item(current)
            .items(all_json.clone())
            .var("nodes", nodes_scope.clone())
            .var("run", json!({ "id": base.run_id, "flowId": base.flow_id, "trigger": base.trigger }))
            .var("now", json!(inner.services.clock.now().to_rfc3339()));
        let settings = match render_value(&node.settings, &scope) {
            Ok(s) => s,
            Err(e) => return CallResult::Failed { error: e.to_string(), retryable: false, input: main },
        };
        let info = RunInfo { node_id: node.id.clone(), node_name: node.name.clone(), item_index: idx, ..base.clone() };
        match call_with_retry(&inner, &services, exec.as_ref(), info, &node, settings, node_input, &cancel, decision)
            .await
        {
            Ok(out) => {
                partial.cost_usd += out.cost_usd;
                match out.waiting_for {
                    Some(approval_id) => {
                        partial.waiting.insert(approval_id, idx);
                    }
                    None => {
                        partial.results.insert(idx, out.ports);
                    }
                }
            }
            Err(e) => return CallResult::Failed { error: e.to_string(), retryable: e.is_retryable(), input: main },
        }
    }

    if !partial.waiting.is_empty() {
        return CallResult::Waiting(partial);
    }
    let ms = started.elapsed().as_millis() as u64;
    CallResult::Done { ports: partial.merged(), cost_usd: partial.cost_usd, ms, input: main }
}

#[allow(clippy::too_many_arguments)]
async fn call_with_retry(
    inner: &Arc<Inner>,
    services: &Services,
    exec: &dyn NodeExecutor,
    info: RunInfo,
    node: &Node,
    settings: Value,
    input: NodeInput,
    cancel: &CancellationToken,
    resume: Option<ApprovalDecision>,
) -> Result<NodeOutput, NodeError> {
    let max_attempts = inner.config.retries + 1;
    let mut attempt = 1;
    loop {
        let mut ctx = NodeCtx {
            settings: settings.clone(),
            run: RunInfo { attempt, ..info.clone() },
            services: services.clone(),
            credential_ids: node.credentials.clone(),
            cancel: cancel.clone(),
            progress: Arc::new(BusProgress { inner: inner.clone() }),
            approvals: Arc::new(EngineGate {
                store: inner.services.store.clone(),
                notifier: inner.services.notifier.clone(),
                clock: inner.services.clock.clone(),
            }),
            resume: resume.clone(),
        };
        let result = tokio::select! {
            _ = cancel.cancelled() => Err(NodeError::Cancelled),
            r = tokio::time::timeout(inner.config.node_timeout(), exec.execute(&mut ctx, input.clone())) => {
                r.unwrap_or(Err(NodeError::Timeout))
            }
        };
        match result {
            Err(e) if e.is_retryable() && attempt < max_attempts => {
                let delay = inner.config.retry_backoff_ms.saturating_mul(1 << (attempt - 1).min(10));
                tracing::warn!(run_id = %info.run_id, node_id = %info.node_id, attempt, error = %e, "retrying node");
                tokio::select! {
                    _ = cancel.cancelled() => return Err(NodeError::Cancelled),
                    _ = inner.services.clock.sleep(Duration::from_millis(delay)) => {}
                }
                attempt += 1;
            }
            other => return other,
        }
    }
}

/// Forwards `ctx.emit_progress` to the event bus as `node.progress`.
struct BusProgress {
    inner: Arc<Inner>,
}

impl ProgressSink for BusProgress {
    fn progress(&self, run: &RunInfo, pct: f32, message: &str) {
        let bus = self.inner.bus.clone();
        let (run_id, node_id, message) = (run.run_id.clone(), run.node_id.clone(), message.to_string());
        tokio::spawn(async move {
            bus.emit(&run_id, Some(&node_id), RunEventKind::NodeProgress { pct, message }).await;
        });
    }
}
