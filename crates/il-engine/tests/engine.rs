//! Engine behaviour with fake nodes: graph execution, retries, timeouts, cancellation,
//! approvals (including across a restart), caps, invoke and replay.

mod support;

use il_core::{ApprovalDecision, ApprovalStatus, Decision, Run, RunEventKind, RunStatus};
use il_engine::Engine;
use il_sdk::services::{FlowRunner, StartRun, Store};
use serde_json::json;
use support::*;

fn start(items: Vec<il_core::Item>) -> StartRun {
    StartRun { flow_id: "flw_t".into(), version: None, trigger: "manual".into(), items, trigger_node: None }
}

async fn run_to_end(engine: &Engine, store: &dyn Store, items: Vec<il_core::Item>) -> (Run, Vec<il_core::RunEvent>) {
    let id = engine.start(start(items)).await.unwrap();
    engine.wait(&id).await;
    let run = store.get_run(&id).await.unwrap().unwrap();
    let events = store.run_events(&id, 0).await.unwrap();
    (run, events)
}

fn outputs(run: &Run, node: &str, port: &str) -> Vec<serde_json::Value> {
    run.state["outputs"][node][port]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .map(|i| i["json"].clone())
        .collect()
}

#[tokio::test]
async fn linear_per_item_flow_renders_expressions_and_orders_events() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("d", "test.double", json!({})),
            ("x", "test.text", json!({ "text": "n={{ item.n }} of {{ items | length }}, first={{ nodes.t.json.n }}" })),
        ],
        &[("t", "out", "d", "in"), ("d", "out", "x", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let (run, events) = run_to_end(&engine, fakes.store.as_ref(), items(&[1, 2, 3])).await;
    assert_eq!(run.status, RunStatus::Succeeded);
    let texts: Vec<_> = outputs(&run, "x", "out").iter().map(|j| j["text"].clone()).collect();
    assert_eq!(texts, [json!("n=2 of 3, first=1"), json!("n=4 of 3, first=1"), json!("n=6 of 3, first=1")]);
    let seqs: Vec<u64> = events.iter().map(|e| e.seq).collect();
    assert_eq!(seqs, (1..=seqs.len() as u64).collect::<Vec<_>>(), "seq is dense and ordered");
    assert!(matches!(events.first().unwrap().kind, RunEventKind::RunStarted { .. }));
    assert!(matches!(events.last().unwrap().kind, RunEventKind::RunSucceeded { .. }));
    let node_runs = fakes.store.node_runs(&run.id).await.unwrap();
    assert_eq!(node_runs.len(), 2, "d and x recorded");
}

#[tokio::test]
async fn branches_skip_untaken_paths_and_merge_waits_for_all() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("b", "test.branch", json!({})),
            ("e", "test.double", json!({})),
            ("o", "test.double", json!({})),
            ("m", "test.collect", json!({})),
        ],
        &[
            ("t", "out", "b", "in"),
            ("b", "even", "e", "in"),
            ("b", "odd", "o", "in"),
            ("e", "out", "m", "a"),
            ("o", "out", "m", "b"),
        ],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1, 2, 3, 4])).await;
    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!(outputs(&run, "m", "out"), [json!({ "sum": 20, "count": 4 })]);

    // Only even numbers: the odd branch is skipped, the merge still runs with what arrived.
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[2, 4])).await;
    assert_eq!(run.state["skipped"], json!(["o"]));
    assert_eq!(outputs(&run, "m", "out"), [json!({ "sum": 12, "count": 2 })]);
}

#[tokio::test]
async fn parallel_branches_respect_the_concurrency_limit() {
    let (fakes, svc) = services();
    let (engine, slow) = engine_with(svc, il_engine::EngineConfig { max_parallel: 2, ..config() });
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("s1", "test.slow", json!({ "ms": 80 })),
            ("s2", "test.slow", json!({ "ms": 80 })),
            ("s3", "test.slow", json!({ "ms": 80 })),
        ],
        &[("t", "out", "s1", "in"), ("t", "out", "s2", "in"), ("t", "out", "s3", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!(slow.max_concurrent.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[tokio::test]
async fn retries_retryable_errors_then_fails() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let ok = flow(
        &[("t", "test.trigger", json!({})), ("f", "test.flaky", json!({ "failures": 2 }))],
        &[("t", "out", "f", "in")],
    );
    fakes.store.put_flow(&ok).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Succeeded, "2 failures fit in 2 retries");

    let bad = flow(
        &[("t", "test.trigger", json!({})), ("f", "test.flaky", json!({ "failures": 3 }))],
        &[("t", "out", "f", "in")],
    );
    fakes.store.put_flow(&bad).await.unwrap();
    let (run, events) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Failed);
    assert!(events.iter().any(|e| matches!(&e.kind, RunEventKind::NodeFailed { retryable: true, .. })));
    assert!(matches!(events.last().unwrap().kind, RunEventKind::RunFailed { .. }));
}

#[tokio::test]
async fn node_timeout_fails_the_run() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, il_engine::EngineConfig { retries: 0, node_timeout_ms: 30, ..config() });
    let f =
        flow(&[("t", "test.trigger", json!({})), ("s", "test.slow", json!({ "ms": 500 }))], &[("t", "out", "s", "in")]);
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Failed);
    assert_eq!(run.error.as_deref(), Some("timed out"));
}

#[tokio::test]
async fn cancel_stops_a_running_flow() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(
        &[("t", "test.trigger", json!({})), ("s", "test.slow", json!({ "ms": 1500 }))],
        &[("t", "out", "s", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let id = engine.start(start(items(&[1]))).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    engine.cancel(&id).await.unwrap();
    assert_eq!(engine.wait(&id).await, RunStatus::Cancelled);
    let events = fakes.store.run_events(&id, 0).await.unwrap();
    assert!(matches!(events.last().unwrap().kind, RunEventKind::RunCancelled {}));
}

#[tokio::test]
async fn approvals_pause_persist_and_resume_after_restart() {
    let (fakes, svc) = services();
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("a", "test.approve", json!({})),
            ("yes", "test.double", json!({})),
            ("no", "test.text", json!({ "text": "rejected" })),
        ],
        &[("t", "out", "a", "in"), ("a", "approved", "yes", "in"), ("a", "rejected", "no", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let id = {
        let (engine, _) = engine_with(svc.clone(), config());
        let id = engine.start(start(items(&[1, 2]))).await.unwrap();
        assert_eq!(engine.wait(&id).await, RunStatus::Waiting);
        id
    }; // first engine dropped: the "restart"

    let pending = fakes.store.list_approvals("ws_t", true).await.unwrap();
    assert_eq!(pending.len(), 2);
    assert_eq!(fakes.notifier.sent.lock().unwrap().iter().filter(|n| n.channel == "slack").count(), 2);
    let waiting = fakes.store.run_events(&id, 0).await.unwrap();
    assert_eq!(waiting.iter().filter(|e| matches!(e.kind, RunEventKind::NodeWaiting { .. })).count(), 2);

    let (engine, _) = engine_with(svc, config());
    let first = pending.iter().find(|a| a.payload["n"] == 1).unwrap().id.clone();
    let second = pending.iter().find(|a| a.payload["n"] == 2).unwrap().id.clone();
    let edited = ApprovalDecision { decision: Decision::Approve, payload: Some(json!({ "n": 10 })), comment: None };
    engine.decide(&first, edited, Some("dana".into())).await.unwrap();
    assert_eq!(engine.wait(&id).await, RunStatus::Waiting, "one approval still pending");
    let reject = ApprovalDecision { decision: Decision::Reject, payload: None, comment: None };
    engine.decide(&second, reject, None).await.unwrap();
    assert_eq!(engine.wait(&id).await, RunStatus::Succeeded);

    let run = fakes.store.get_run(&id).await.unwrap().unwrap();
    assert_eq!(outputs(&run, "yes", "out"), [json!({ "n": 20 })], "edited payload flows on");
    assert_eq!(outputs(&run, "no", "out"), [json!({ "n": 2, "text": "rejected" })]);
    let a = fakes.store.get_approval(&first).await.unwrap().unwrap();
    assert_eq!(a.status, ApprovalStatus::Approved);
    assert_eq!(a.decided_by.as_deref(), Some("dana"));
    let events = fakes.store.run_events(&id, 0).await.unwrap();
    assert_eq!(
        events.iter().filter(|e| matches!(e.kind, RunEventKind::NodeWaiting { .. })).count(),
        2,
        "no duplicate waits"
    );
    assert_eq!(events.iter().filter(|e| matches!(e.kind, RunEventKind::ApprovalDecided { .. })).count(), 2);
    // Deciding twice is refused.
    let again = ApprovalDecision { decision: Decision::Approve, payload: None, comment: None };
    assert!(engine.decide(&first, again, None).await.is_err());
}

#[tokio::test]
async fn b03b_expiry_approves_after_restart_and_rejects_late_manual_decision() {
    let fakes = il_sdk::testkit::FakeServices::default();
    let svc = fakes.services();
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("a", "test.approve", json!({"timeoutHours": 2.0, "onTimeout": "approve"})),
            ("yes", "test.double", json!({})),
        ],
        &[("t", "out", "a", "in"), ("a", "approved", "yes", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let run_id = {
        let (engine, _) = engine_with(svc.clone(), config());
        let id = engine.start(start(items(&[3]))).await.unwrap();
        assert_eq!(engine.wait(&id).await, RunStatus::Waiting);
        id
    };
    let approval = fakes.store.list_approvals("ws_t", true).await.unwrap().pop().unwrap();
    assert_eq!(approval.on_timeout, Some(il_core::ApprovalTimeout::Approve));
    assert_eq!(approval.expires_at, Some(approval.created_at + chrono::Duration::hours(2)));
    fakes.clock.advance(std::time::Duration::from_secs(2 * 3600));
    let (restarted, _) = engine_with(svc, config());
    let late = ApprovalDecision { decision: Decision::Reject, payload: None, comment: None };
    assert!(restarted.decide(&approval.id, late, Some("late".into())).await.is_err());
    assert_eq!(restarted.expire_due_approvals().await.unwrap(), 1);
    assert_eq!(restarted.wait(&run_id).await, RunStatus::Succeeded);
    let decided = fakes.store.get_approval(&approval.id).await.unwrap().unwrap();
    assert_eq!(decided.status, ApprovalStatus::Approved);
    assert_eq!(decided.decided_by.as_deref(), Some("timeout"));
    let run = fakes.store.get_run(&run_id).await.unwrap().unwrap();
    assert_eq!(outputs(&run, "yes", "out"), [json!({"n":6})]);
    assert_eq!(restarted.expire_due_approvals().await.unwrap(), 0);
}

#[tokio::test]
async fn b03b_reject_and_keep_waiting_timeout_policies() {
    for (policy, expected_port) in [("reject", "rejected"), ("keepWaiting", "approved")] {
        let fakes = il_sdk::testkit::FakeServices::default();
        let svc = fakes.services();
        let f = flow(
            &[
                ("t", "test.trigger", json!({})),
                ("a", "test.approve", json!({"timeoutHours": 1.0, "onTimeout": policy})),
                ("yes", "test.double", json!({})),
                ("no", "test.text", json!({"text":"no"})),
            ],
            &[("t", "out", "a", "in"), ("a", "approved", "yes", "in"), ("a", "rejected", "no", "in")],
        );
        fakes.store.put_flow(&f).await.unwrap();
        let (engine, _) = engine_with(svc, config());
        let id = engine.start(start(items(&[1]))).await.unwrap();
        assert_eq!(engine.wait(&id).await, RunStatus::Waiting);
        let approval = fakes.store.list_approvals("ws_t", true).await.unwrap().pop().unwrap();
        fakes.clock.advance(std::time::Duration::from_secs(3600));
        let expired = engine.expire_due_approvals().await.unwrap();
        if policy == "keepWaiting" {
            assert_eq!(expired, 0);
            let manual = ApprovalDecision { decision: Decision::Approve, payload: None, comment: None };
            engine.decide(&approval.id, manual, Some("human".into())).await.unwrap();
        } else {
            assert_eq!(expired, 1);
            assert!(
                engine
                    .decide(
                        &approval.id,
                        ApprovalDecision { decision: Decision::Approve, payload: None, comment: None },
                        None
                    )
                    .await
                    .is_err()
            );
        }
        assert_eq!(engine.wait(&id).await, RunStatus::Succeeded);
        let run = fakes.store.get_run(&id).await.unwrap().unwrap();
        assert_eq!(outputs(&run, if expected_port == "approved" { "yes" } else { "no" }, "out").len(), 1);
    }
}

#[tokio::test]
async fn b03b_expiry_does_not_resume_a_cancelled_run() {
    let fakes = il_sdk::testkit::FakeServices::default();
    let svc = fakes.services();
    let f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("a", "test.approve", json!({"timeoutHours": 1.0, "onTimeout": "approve"})),
        ],
        &[("t", "out", "a", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let (engine, _) = engine_with(svc, config());
    let id = engine.start(start(items(&[1]))).await.unwrap();
    assert_eq!(engine.wait(&id).await, RunStatus::Waiting);
    let approval = fakes.store.list_approvals("ws_t", true).await.unwrap().pop().unwrap();
    engine.cancel(&id).await.unwrap();
    fakes.clock.advance(std::time::Duration::from_secs(3600));
    assert_eq!(engine.expire_due_approvals().await.unwrap(), 0);
    assert_eq!(fakes.store.get_run(&id).await.unwrap().unwrap().status, RunStatus::Cancelled);
    assert_eq!(fakes.store.get_approval(&approval.id).await.unwrap().unwrap().status, ApprovalStatus::Pending);
}

#[tokio::test]
async fn spend_cap_pauses_the_run() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let mut f = flow(
        &[
            ("t", "test.trigger", json!({})),
            ("c1", "test.cost", json!({ "usd": 0.6 })),
            ("c2", "test.cost", json!({ "usd": 0.6 })),
            ("c3", "test.cost", json!({ "usd": 0.6 })),
        ],
        &[("t", "out", "c1", "in"), ("c1", "out", "c2", "in"), ("c2", "out", "c3", "in")],
    );
    f.settings.spend_cap_usd_per_run = Some(1.0);
    fakes.store.put_flow(&f).await.unwrap();
    let (run, events) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::CapReached);
    assert!((run.cost_usd - 1.2).abs() < 1e-9);
    let cap = events.iter().find_map(|e| match &e.kind {
        RunEventKind::RunCapReached { cap, limit_usd, .. } => Some((cap.clone(), *limit_usd, e.node_id.clone())),
        _ => None,
    });
    assert_eq!(cap, Some(("run".into(), 1.0, Some("c3".into()))));
}

#[tokio::test]
async fn invoke_returns_the_respond_output() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(
        &[("t", "test.trigger", json!({})), ("r", "output.respond", json!({ "body": "Hi {{ item.name }}" }))],
        &[("t", "out", "r", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();
    let req = StartRun { items: vec![il_core::Item::json(json!({ "name": "Maya" }))], ..start(vec![]) };
    let out = engine.invoke(req).await.unwrap();
    assert_eq!(out.run.status, RunStatus::Succeeded);
    assert_eq!(out.response.len(), 1);
    assert_eq!(out.response[0].json, json!({ "answer": "Hi Maya" }));
}

#[tokio::test]
async fn failures_unknown_types_and_disabled_nodes() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(&[("t", "test.trigger", json!({})), ("x", "nope.missing", json!({}))], &[("t", "out", "x", "in")]);
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Failed);
    assert!(run.error.unwrap().contains("unknown node type"));

    let mut f = flow(&[("t", "test.trigger", json!({})), ("x", "test.fail", json!({}))], &[("t", "out", "x", "in")]);
    f.nodes[1].disabled = true;
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[7])).await;
    assert_eq!(run.status, RunStatus::Succeeded, "disabled nodes pass items through");
    assert_eq!(outputs(&run, "x", "out"), [json!({ "n": 7 })]);
}

#[tokio::test]
async fn replays_a_recorded_run_with_a_replay_label() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let f = flow(&[("t", "test.trigger", json!({})), ("d", "test.double", json!({}))], &[("t", "out", "d", "in")]);
    fakes.store.put_flow(&f).await.unwrap();
    let (recorded, original) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    let mut live = engine.events().subscribe();
    let id = engine.replay(&recorded.id).await.unwrap();
    assert_eq!(engine.wait(&id).await, RunStatus::Succeeded);
    let replayed = fakes.store.run_events(&id, 0).await.unwrap();
    assert_eq!(replayed.len(), original.len());
    assert!(fakes.store.get_run(&id).await.unwrap().unwrap().replay);
    let first_live = live.recv().await.unwrap();
    assert_eq!(first_live.run_id, id, "replayed events are broadcast live");
}

/// An agent stand-in that outputs the names of the tools it was given.
struct ListTools(il_core::NodeManifest);

#[async_trait::async_trait]
impl il_sdk::NodeExecutor for ListTools {
    fn manifest(&self) -> &il_core::NodeManifest {
        &self.0
    }
    async fn execute(
        &self,
        ctx: &mut il_sdk::ctx::NodeCtx,
        _input: il_sdk::NodeInput,
    ) -> Result<il_sdk::NodeOutput, il_sdk::NodeError> {
        let names: Vec<_> = ctx.tools().iter().map(|t| t.name().to_string()).collect();
        Ok(il_sdk::NodeOutput::out(vec![il_core::Item::json(json!({ "tools": names }))]))
    }
}

struct NamedTool(&'static str);

#[async_trait::async_trait]
impl il_sdk::services::AgentTool for NamedTool {
    fn name(&self) -> &str {
        self.0
    }
    fn description(&self) -> &str {
        "test tool"
    }
    fn schema(&self) -> serde_json::Value {
        json!({ "type": "object" })
    }
    async fn invoke(&self, _args: serde_json::Value) -> Result<serde_json::Value, il_sdk::services::ToolError> {
        Ok(json!({}))
    }
}

/// Gives flows in `ws_t` a `lookup_customer` tool.
struct WorkspaceTools;

#[async_trait::async_trait]
impl il_sdk::services::FlowToolProvider for WorkspaceTools {
    async fn for_flow(
        &self,
        workspace_id: &str,
        _flow: &il_core::Flow,
    ) -> Result<Vec<std::sync::Arc<dyn il_sdk::services::AgentTool>>, il_sdk::services::ToolError> {
        Ok(if workspace_id == "ws_t" { vec![std::sync::Arc::new(NamedTool("lookup_customer"))] } else { vec![] })
    }
}

#[tokio::test]
async fn agents_get_the_flows_own_tools_when_a_run_starts() {
    let (fakes, mut svc) = services();
    svc.flow_tools = std::sync::Arc::new(WorkspaceTools);
    let (lookup, _) = lookup();
    let lookup = lookup.with(std::sync::Arc::new(ListTools(m("ai.agent"))));
    let engine = Engine::new(svc, lookup, config());
    let f = flow(&[("t", "test.trigger", json!({})), ("a", "ai.agent", json!({}))], &[("t", "out", "a", "in")]);
    fakes.store.put_flow(&f).await.unwrap();
    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[1])).await;
    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!(outputs(&run, "a", "out")[0]["tools"], json!(["lookup_customer"]));
}

#[tokio::test]
async fn test_runs_without_items_use_the_triggers_sample_items() {
    let (fakes, svc) = services();
    let (engine, _) = engine_with(svc, config());
    let samples = json!({ "sampleItems": [{ "n": 4 }, { "n": 5 }] });
    let f = flow(&[("t", "test.trigger", samples), ("d", "test.double", json!({}))], &[("t", "out", "d", "in")]);
    fakes.store.put_flow(&f).await.unwrap();
    let id = engine.start(StartRun { trigger: "test".into(), ..start(vec![]) }).await.unwrap();
    engine.wait(&id).await;
    let run = fakes.store.get_run(&id).await.unwrap().unwrap();
    assert_eq!(outputs(&run, "d", "out"), [json!({ "n": 8 }), json!({ "n": 10 })]);
    let (manual, _) = run_to_end(&engine, fakes.store.as_ref(), vec![]).await;
    assert!(outputs(&manual, "d", "out").is_empty(), "only test runs fall back to the samples");
}

/// A step with outside effects that counts how often it really runs.
struct Sends(il_core::NodeManifest, std::sync::atomic::AtomicUsize);

#[async_trait::async_trait]
impl il_sdk::NodeExecutor for Sends {
    fn manifest(&self) -> &il_core::NodeManifest {
        &self.0
    }
    async fn execute(
        &self,
        _ctx: &mut il_sdk::ctx::NodeCtx,
        input: il_sdk::NodeInput,
    ) -> Result<il_sdk::NodeOutput, il_sdk::NodeError> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(il_sdk::NodeOutput::out(input.main().to_vec()))
    }
}

#[tokio::test]
async fn test_runs_preview_outside_effects_instead_of_running_them() {
    let (fakes, svc) = services();
    let mut manifest = m("test.send");
    manifest.side_effect = il_core::SideEffect::External;
    let sends = std::sync::Arc::new(Sends(manifest, Default::default()));
    let (lookup, _) = lookup();
    let engine = Engine::new(svc, lookup.with(sends.clone()), config());
    let f = flow(
        &[("t", "test.trigger", json!({})), ("s", "test.send", json!({ "to": "{{ item.n }}@example.com" }))],
        &[("t", "out", "s", "in")],
    );
    fakes.store.put_flow(&f).await.unwrap();

    let test = StartRun { trigger: "test".into(), ..start(items(&[7])) };
    let id = engine.start(test).await.unwrap();
    engine.wait(&id).await;
    let run = fakes.store.get_run(&id).await.unwrap().unwrap();
    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!(sends.1.load(std::sync::atomic::Ordering::SeqCst), 0, "a test run never sends");
    let out = outputs(&run, "s", "out");
    assert_eq!(out[0]["n"], 7);
    assert_eq!(out[0]["testRun"]["would"]["to"], "7@example.com");

    let (run, _) = run_to_end(&engine, fakes.store.as_ref(), items(&[7])).await;
    assert_eq!(run.status, RunStatus::Succeeded);
    assert_eq!(sends.1.load(std::sync::atomic::Ordering::SeqCst), 1, "a real run does");
}
