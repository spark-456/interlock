//! The graph driver: runs ready nodes (all upstream nodes resolved) with a concurrency limit,
//! applies results to the checkpoint, persists after every node, and finishes the run.

use std::collections::BTreeSet;
use std::sync::Arc;

use il_core::{Flow, Item, Node, NodeRun, Run, RunEventKind, RunStatus};
use il_sdk::Services;
use il_sdk::ctx::RunInfo;
use il_sdk::services::store::{SpendEntry, SpendScope};
use serde_json::{Map, Value, json};
use tokio::task::JoinSet;

use super::Inner;
use super::invoke::{CallResult, Resume, run_node};
use crate::caps;
use crate::state::{Checkpoint, Ports};

/// The type id whose outputs become the run's response (chat/API/MCP invocations).
pub(crate) const RESPOND: &str = "output.respond";

/// Drives a run until it finishes, pauses or hits a cap. Optionally resumes one paused node.
pub(crate) async fn drive(
    inner: Arc<Inner>,
    mut run: Run,
    flow: Flow,
    mut cp: Checkpoint,
    resume: Option<(String, Resume)>,
) {
    let cancel = inner.cancel_token(&run.id);
    run.status = RunStatus::Running;
    inner.set_status(&run.id, RunStatus::Running);
    persist(&inner, &mut run, &cp).await;
    let base = RunInfo {
        run_id: run.id.clone(),
        flow_id: flow.id.clone(),
        workspace_id: run.workspace_id.clone(),
        trigger: run.trigger.clone(),
        test: run.trigger == "test" || run.replay,
        ..Default::default()
    };

    let services = run_services(&inner, &flow, &run.workspace_id).await;
    let mut tasks: JoinSet<(String, CallResult)> = JoinSet::new();
    let mut running: BTreeSet<String> = BTreeSet::new();
    if let Some((node_id, r)) = resume
        && let Some(node) = flow.node(&node_id).cloned()
    {
        running.insert(node_id.clone());
        let (inner2, base2, scope, cancel2) = (inner.clone(), base.clone(), nodes_scope(&cp), cancel.clone());
        let (input, services2) = (r.partial.input.clone(), services.clone());
        tasks.spawn(async move {
            (node_id, run_node(inner2, services2, base2, node, input, scope, cancel2, Some(r)).await)
        });
    }

    loop {
        if cancel.is_cancelled() {
            tasks.abort_all();
            return finish(&inner, &mut run, &cp, RunStatus::Cancelled, None).await;
        }
        // Start every ready node, up to the concurrency limit.
        let mut progressed = true;
        while progressed {
            progressed = false;
            for node in &flow.nodes {
                if running.len() >= inner.config.max_parallel.max(1) {
                    break;
                }
                if cp.resolved(&node.id) || cp.partial.contains_key(&node.id) || running.contains(&node.id) {
                    continue;
                }
                let incoming: Vec<_> = flow.incoming(&node.id).filter(|e| flow.node(&e.from.node).is_some()).collect();
                if incoming.is_empty() {
                    // Other roots: extra triggers are not taken; source nodes (knowledge bases…) run.
                    let is_trigger = inner.nodes.get(&node.node_type).is_some_and(|n| n.manifest().trigger);
                    if is_trigger {
                        cp.skipped.insert(node.id.clone());
                        progressed = true;
                        continue;
                    }
                } else if !incoming.iter().all(|e| cp.resolved(&e.from.node)) {
                    continue;
                }
                let mut input = Ports::new();
                for e in &incoming {
                    let items =
                        cp.outputs.get(&e.from.node).and_then(|p| p.get(&e.from.port)).cloned().unwrap_or_default();
                    input.entry(e.to.port.clone()).or_default().extend(items);
                }
                if !incoming.is_empty() && input.values().all(Vec::is_empty) {
                    cp.skipped.insert(node.id.clone());
                    progressed = true;
                    continue;
                }
                if let Some(hit) = caps::check(
                    inner.services.store.as_ref(),
                    inner.services.clock.now(),
                    &run,
                    &flow,
                    &inner.config.caps,
                )
                .await
                {
                    tasks.abort_all();
                    let kind = RunEventKind::RunCapReached {
                        cap: hit.cap.into(),
                        limit_usd: hit.limit_usd,
                        spent_usd: hit.spent_usd,
                    };
                    inner.bus.emit(&run.id, Some(&node.id), kind).await;
                    return finish(&inner, &mut run, &cp, RunStatus::CapReached, None).await;
                }
                inner.bus.emit(&run.id, Some(&node.id), RunEventKind::NodeStarted {}).await;
                running.insert(node.id.clone());
                progressed = true;
                let (inner2, base2, scope, cancel2, node2) =
                    (inner.clone(), base.clone(), nodes_scope(&cp), cancel.clone(), node.clone());
                let (id, services2) = (node.id.clone(), services.clone());
                tasks.spawn(async move {
                    (id, run_node(inner2, services2, base2, node2, input, scope, cancel2, None).await)
                });
            }
        }

        let next = tokio::select! {
            _ = cancel.cancelled() => continue,
            next = tasks.join_next() => next,
        };
        let Some(joined) = next else { break };
        let Ok((node_id, result)) = joined else { continue };
        running.remove(&node_id);
        let Some(node) = flow.node(&node_id).cloned() else { continue };
        match result {
            CallResult::Done { ports, cost_usd, ms, input } => {
                complete(&inner, &run, &node, &mut cp, ports, cost_usd, ms, input).await;
            }
            CallResult::Waiting(partial) => {
                for approval_id in partial.waiting.keys() {
                    if partial_announced(&cp, &node_id, approval_id) {
                        continue;
                    }
                    let kind = RunEventKind::NodeWaiting { approval_id: approval_id.clone() };
                    inner.bus.emit(&run.id, Some(&node_id), kind).await;
                }
                cp.partial.insert(node_id.clone(), partial);
            }
            CallResult::Failed { error, retryable, input } => {
                tasks.abort_all();
                inner
                    .bus
                    .emit(&run.id, Some(&node_id), RunEventKind::NodeFailed { error: error.clone(), retryable })
                    .await;
                let nr = node_run(&run, &node_id, RunStatus::Failed, input, Ports::new(), Some(error.clone()), 0, 0.0);
                let _ = inner.services.store.put_node_run(&nr).await;
                return finish(&inner, &mut run, &cp, RunStatus::Failed, Some(error)).await;
            }
        }
        persist(&inner, &mut run, &cp).await;
    }

    let status = if cp.partial.is_empty() { RunStatus::Succeeded } else { RunStatus::Waiting };
    finish(&inner, &mut run, &cp, status, None).await;
}

fn partial_announced(cp: &Checkpoint, node_id: &str, approval_id: &str) -> bool {
    cp.partial.get(node_id).is_some_and(|p| p.waiting.contains_key(approval_id))
}

/// The services a run's steps see: the engine's, plus the flow's own tools for its agents.
async fn run_services(inner: &Inner, flow: &Flow, workspace_id: &str) -> Services {
    let mut services = inner.services.clone();
    if flow.nodes.iter().any(|n| n.node_type == "ai.agent") {
        match services.flow_tools.for_flow(workspace_id, flow).await {
            Ok(tools) => services.tools.extend(tools),
            Err(e) => tracing::warn!(flow_id = %flow.id, error = %e, "could not load the flow's tools"),
        }
    }
    services
}

#[allow(clippy::too_many_arguments)]
async fn complete(
    inner: &Arc<Inner>,
    run: &Run,
    node: &Node,
    cp: &mut Checkpoint,
    ports: Ports,
    cost_usd: f64,
    ms: u64,
    input: Vec<Item>,
) {
    let mut count = 0u64;
    for (port, items) in &ports {
        count += items.len() as u64;
        let preview: Vec<Item> = items.iter().take(3).cloned().collect();
        let kind = RunEventKind::NodeItems { port: port.clone(), items: preview, total: items.len() as u64 };
        inner.bus.emit(&run.id, Some(&node.id), kind).await;
    }
    inner.bus.emit(&run.id, Some(&node.id), RunEventKind::NodeSucceeded { count, ms, cost_usd }).await;
    if cost_usd > 0.0 {
        let entry = SpendEntry {
            workspace_id: run.workspace_id.clone(),
            flow_id: Some(run.flow_id.clone()),
            run_id: Some(run.id.clone()),
            build_id: None,
            role: "node".into(),
            cost_usd,
            at: inner.services.clock.now(),
        };
        let _ = inner.services.store.record_spend(&entry).await;
    }
    if node.node_type == RESPOND {
        cp.response.extend(ports.values().flatten().cloned());
    }
    let nr = node_run(run, &node.id, RunStatus::Succeeded, input, ports.clone(), None, ms, cost_usd);
    let _ = inner.services.store.put_node_run(&nr).await;
    cp.partial.remove(&node.id);
    cp.outputs.insert(node.id.clone(), ports);
}

#[allow(clippy::too_many_arguments)]
fn node_run(
    run: &Run,
    node_id: &str,
    status: RunStatus,
    input: Vec<Item>,
    output: Ports,
    error: Option<String>,
    ms: u64,
    cost_usd: f64,
) -> NodeRun {
    NodeRun { run_id: run.id.clone(), node_id: node_id.into(), status, attempt: 1, input, output, error, ms, cost_usd }
}

/// `nodes` expression scope: `{ "<id>": { "json": first item, "items": [...] } }`.
fn nodes_scope(cp: &Checkpoint) -> Value {
    let mut map = Map::new();
    for (id, ports) in &cp.outputs {
        let items: Vec<Value> = ports.values().flatten().map(|i| i.json.clone()).collect();
        let first = ports.get("out").and_then(|v| v.first()).map(|i| i.json.clone()).or_else(|| items.first().cloned());
        map.insert(id.clone(), json!({ "json": first.unwrap_or(Value::Null), "items": items }));
    }
    Value::Object(map)
}

async fn persist(inner: &Inner, run: &mut Run, cp: &Checkpoint) {
    run.state = cp.to_state();
    if let Err(e) = inner.services.store.put_run(run).await {
        tracing::error!(run_id = %run.id, error = %e, "failed to persist run");
    }
}

async fn finish(inner: &Inner, run: &mut Run, cp: &Checkpoint, status: RunStatus, error: Option<String>) {
    let now = inner.services.clock.now();
    run.status = status;
    run.error = error.clone();
    run.cost_usd = inner.services.store.spend(&SpendScope::Run(run.id.clone())).await.unwrap_or(run.cost_usd);
    if status.is_terminal() {
        run.finished_at = Some(now);
    }
    persist(inner, run, cp).await;
    let ms = (now - run.started_at).num_milliseconds().max(0) as u64;
    let kind = match status {
        RunStatus::Succeeded => Some(RunEventKind::RunSucceeded { ms, cost_usd: run.cost_usd }),
        RunStatus::Failed => Some(RunEventKind::RunFailed { error: error.unwrap_or_default() }),
        RunStatus::Cancelled => Some(RunEventKind::RunCancelled {}),
        _ => None,
    };
    if let Some(kind) = kind {
        inner.bus.emit(&run.id, None, kind).await;
    }
    if status.is_terminal()
        && let Ok(mut map) = inner.cancels.lock()
    {
        map.remove(&run.id);
    }
    inner.set_status(&run.id, status);
}
