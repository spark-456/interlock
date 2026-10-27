//! A02: Ask Interlock diffs, test-and-fix rounds during a build, and the AI app kit, with scripted
//! Architect replies and scripted test runs.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use il_architect::Architect;
use il_architect::appkit::missing_guardrails;
use il_architect::ask::{ask, diff};
use il_architect::catalog::Catalog;
use il_architect::compile::{Outcome, compile};
use il_architect::testfix::MAX_FIX_ROUNDS;
use il_core::dto::BuildRequest;
use il_core::{ApprovalDecision, BuildEvent, BuildEventKind, Flow, FlowKind, NodeManifest, NodeRun, RunStatus};
use il_sdk::services::ArchitectService;
use il_sdk::services::models::ChatRole;
use il_sdk::services::runner::{FlowRunner, InvokeOutcome, RunnerError, StartRun};
use il_sdk::services::store::Store;
use il_sdk::testkit::{FakeGateway, FakeRunner, FakeServices};
use serde_json::{Value, json};

const FB: &str =
    "Reply to support emails from our help center; ask me in Slack before refunds; log replies in a sheet.";
const FC: &str = "A support chat agent that answers from these documents and can look up invoices through our API.";

/// The fixture catalog; `ai.guardrails` is left out so the app tests don't depend on it.
fn catalog() -> Catalog {
    let v: Value = serde_json::from_str(include_str!("../../../web/fixtures/api/nodes.json")).unwrap();
    let all: Vec<NodeManifest> = serde_json::from_value(v["nodes"].clone()).unwrap();
    Catalog::new(all.into_iter().filter(|m| m.type_id != "ai.guardrails").collect::<Vec<_>>())
}

fn draft(name: &str) -> Value {
    serde_json::from_str(if name == "fb" { include_str!("golden/fb.json") } else { include_str!("golden/fc.json") })
        .unwrap()
}

/// `draft` with one step's setting changed.
fn with_setting(mut d: Value, node: &str, key: &str, value: &str) -> Value {
    let n = d["nodes"].as_array_mut().unwrap().iter_mut().find(|n| n["id"] == node).unwrap();
    n["settings"][key] = json!(value);
    d
}

async fn compiled(name: &str, goal: &str) -> Flow {
    let gw = FakeGateway::default();
    gw.push_json(draft(name));
    match compile(&gw, &catalog(), goal, goal, false).await.unwrap().outcome {
        Outcome::Flow { flow, .. } => *flow,
        Outcome::Questions(_) => panic!("asked questions"),
    }
}

fn user_messages(gw: &FakeGateway) -> Vec<String> {
    let calls = gw.calls();
    calls
        .iter()
        .flat_map(|(_, r)| r.messages.iter().filter(|m| m.role == ChatRole::User))
        .map(|m| m.content.clone())
        .collect()
}

#[tokio::test]
async fn ask_returns_a_patch_that_reproduces_the_revision() {
    let catalog = catalog();
    let mut flow = compiled("fb", FB).await;
    let reply = flow.nodes.iter_mut().find(|n| n.id == "n_reply").unwrap();
    reply.position.x = 9999.0;
    reply.credentials.insert("gmail".into(), "cred_1".into());

    let mut revised = with_setting(draft("fb"), "n_draft", "instructions", "Write warmly.");
    revised["nodes"].as_array_mut().unwrap().retain(|n| n["id"] != "n_spam");
    revised["edges"].as_array_mut().unwrap().retain(|e| e["to"] != "n_spam");
    let gw = FakeGateway::default();
    gw.push_json(revised);
    let (d, _) = ask(&gw, &catalog, &flow, "Make replies warmer and stop labelling spam").await.unwrap();

    assert_eq!(d.summary, "−1 step, 1 changed");
    let asked = user_messages(&gw).join("\n");
    assert!(asked.contains("Change request: Make replies warmer") && asked.contains("\"n_spam\""));
    let mut after = serde_json::to_value(&flow).unwrap();
    for op in d.json_patch.as_array().unwrap() {
        assert_eq!(op["op"], "replace");
        after[&op["path"].as_str().unwrap()[1..]] = op["value"].clone();
    }
    let after: Flow = serde_json::from_value(after).unwrap();
    assert!(after.node("n_spam").is_none());
    assert_eq!(after.node("n_draft").unwrap().settings["instructions"], "Write warmly.");
    let kept = after.node("n_reply").unwrap();
    assert_eq!((kept.position.x, kept.credentials.get("gmail").map(String::as_str)), (9999.0, Some("cred_1")));
    assert_eq!(after.id, flow.id);
}

#[tokio::test]
async fn diff_summarises_what_changed() {
    let before = compiled("fb", FB).await;
    assert_eq!(diff(&before, &before).summary, "No changes");
    assert_eq!(diff(&before, &before).json_patch, json!([]));

    let mut after = before.clone();
    let mut extra = after.nodes[0].clone();
    extra.id = "n_extra".into();
    after.nodes.push(extra);
    after.nodes[1].name = "Sort the email".into();
    after.nodes[2].position.y += 50.0;
    assert_eq!(diff(&before, &after).summary, "+1 step, 1 changed");

    let mut rewired = before.clone();
    rewired.edges.pop();
    assert_eq!(diff(&before, &rewired).summary, "Connections changed");
}

/// Fails the first `fail` test runs at `node`, then succeeds.
struct FlakyRunner {
    store: Arc<dyn Store>,
    node: &'static str,
    fail: Mutex<usize>,
}

#[async_trait]
impl FlowRunner for FlakyRunner {
    async fn start(&self, _req: StartRun) -> Result<String, RunnerError> {
        Ok("run_x".into())
    }
    async fn invoke(&self, req: StartRun) -> Result<InvokeOutcome, RunnerError> {
        assert_eq!(req.trigger, "test");
        let failing = {
            let mut left = self.fail.lock().unwrap();
            let failing = *left > 0;
            *left = left.saturating_sub(1);
            failing
        };
        let id = il_sdk::id("run");
        if failing {
            let node_run = NodeRun {
                run_id: id.clone(),
                node_id: self.node.into(),
                status: RunStatus::Failed,
                attempt: 1,
                input: vec![],
                output: Default::default(),
                error: Some("the input expression is empty".into()),
                ms: 1,
                cost_usd: 0.0,
            };
            self.store.put_node_run(&node_run).await.unwrap();
        }
        // FakeRunner's run, marked as this one's outcome.
        let mut run = FakeRunner::default().invoke(req).await?.run;
        run.id = id;
        run.status = if failing { RunStatus::Failed } else { RunStatus::Succeeded };
        Ok(InvokeOutcome { run, response: vec![] })
    }
    async fn decide(&self, _id: &str, _d: ApprovalDecision, _by: Option<String>) -> Result<(), RunnerError> {
        Ok(())
    }
    async fn cancel(&self, _run_id: &str) -> Result<(), RunnerError> {
        Ok(())
    }
    async fn replay(&self, _id: &str) -> Result<String, RunnerError> {
        Ok("run_replay".into())
    }
}

async fn build(fakes: &FakeServices, runner: Arc<dyn FlowRunner>, goal: &str) -> Vec<BuildEvent> {
    let architect = Architect::new(fakes.services(), runner, catalog());
    let req = BuildRequest { goal: goal.into(), kind: None, attachments: vec![], hints: vec![] };
    let id = architect.start_build("ws_1", req).await.unwrap();
    for _ in 0..500 {
        let evs = fakes.store.build_events(&id, 0).await.unwrap();
        if evs.iter().any(|e| matches!(e.kind, BuildEventKind::Ready { .. } | BuildEventKind::Failed { .. })) {
            return evs;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the build never ended");
}

fn results(evs: &[BuildEvent]) -> Vec<(u32, u32)> {
    evs.iter()
        .filter_map(|e| match e.kind {
            BuildEventKind::TestResult { passed, failed } => Some((passed, failed)),
            _ => None,
        })
        .collect()
}

async fn saved(fakes: &FakeServices, evs: &[BuildEvent]) -> Flow {
    let Some(BuildEventKind::Ready { flow_id }) = evs.last().map(|e| &e.kind) else { panic!("{evs:#?}") };
    fakes.store.get_flow(flow_id).await.unwrap().unwrap()
}

#[tokio::test]
async fn a_failing_test_run_is_fixed_and_run_again() {
    let fakes = FakeServices::default();
    fakes.models.push_json(draft("fb"));
    fakes.models.push_json(with_setting(draft("fb"), "n_draft", "input", "{{ item.text }}"));
    let runner = Arc::new(FlakyRunner { store: fakes.services().store, node: "n_draft", fail: Mutex::new(1) });
    let evs = build(&fakes, runner, FB).await;

    assert_eq!(results(&evs), [(0, 1), (1, 0)]);
    let fix = evs.iter().find(|e| matches!(e.kind, BuildEventKind::Fix { .. })).unwrap();
    let BuildEventKind::Fix { description, title, source, .. } = &fix.kind else { unreachable!() };
    assert_eq!(fix.node_id.as_deref(), Some("n_draft"));
    assert_eq!((description.as_str(), source.as_deref()), ("the input expression is empty", Some("Test run")));
    assert!(title.as_deref().unwrap().starts_with("Fixed "));
    assert!(
        evs.iter()
            .any(|e| matches!(&e.kind, BuildEventKind::FlowPatch { json_patch } if json_patch[0]["path"] == "/nodes"))
    );
    assert!(user_messages(&fakes.models).iter().any(|m| m.contains("step `n_draft`: the input expression is empty")));
    assert_eq!(saved(&fakes, &evs).await.node("n_draft").unwrap().settings["input"], "{{ item.text }}");
}

#[tokio::test]
async fn gives_up_fixing_after_the_last_round_but_keeps_the_flow() {
    let fakes = FakeServices::default();
    fakes.models.push_json(draft("fb"));
    for _ in 0..MAX_FIX_ROUNDS {
        fakes.models.push_json(draft("fb"));
    }
    let runner = Arc::new(FlakyRunner { store: fakes.services().store, node: "n_draft", fail: Mutex::new(usize::MAX) });
    let evs = build(&fakes, runner, FB).await;

    assert_eq!(results(&evs), vec![(0, 1); MAX_FIX_ROUNDS + 1]);
    assert_eq!(fakes.models.calls().len(), MAX_FIX_ROUNDS + 1);
    assert!(evs.iter().any(|e| matches!(&e.kind, BuildEventKind::Log { text } if text.contains("remain after"))));
    saved(&fakes, &evs).await;
}

#[tokio::test]
async fn an_app_runs_its_suite_and_fixes_failed_answers() {
    let fakes = FakeServices::default();
    let m = &fakes.models;
    m.push_json(draft("fc"));
    m.push_json(json!({"cases": [
        {"kind": "knowledge", "input": "What is the refund window?", "expect": "says 30 days"},
        {"kind": "tool", "input": "What is the status of invoice 42?", "expect": "looks the invoice up"}]}));
    m.push_json(json!({"pass": true}));
    m.push_json(json!({"pass": false, "reason": "it did not look anything up"}));
    m.push_json(with_setting(draft("fc"), "n_billing", "instructions", "Always look invoices up with the tool."));
    m.push_json(json!({"pass": true}));
    m.push_json(json!({"pass": true}));
    let runner = Arc::new(FakeRunner::default());
    let evs = build(&fakes, runner.clone(), FC).await;

    assert_eq!(results(&evs), [(1, 1), (2, 0)]);
    let sources: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.kind {
            BuildEventKind::Fix { source, .. } => source.clone(),
            _ => None,
        })
        .collect();
    assert_eq!(sources, ["Test suite"]);
    let asked: Vec<_> = runner.started().iter().map(|r| r.items[0].json["message"].clone()).collect();
    assert_eq!(asked.len(), 4);
    assert_eq!(asked[1], "What is the status of invoice 42?");
    let flow = saved(&fakes, &evs).await;
    assert_eq!(flow.kind, FlowKind::App);
    assert_eq!(flow.node("n_billing").unwrap().settings["instructions"], "Always look invoices up with the tool.");
}

#[tokio::test]
async fn an_app_without_safety_rules_is_flagged_when_the_catalog_has_them() {
    let flow = compiled("fc", FC).await;
    assert_eq!(missing_guardrails(&flow, &catalog()), None);
    let v: Value = serde_json::from_str(include_str!("../../../web/fixtures/api/nodes.json")).unwrap();
    let mut manifests: Vec<NodeManifest> = serde_json::from_value(v["nodes"].clone()).unwrap();
    let mut guard = manifests[0].clone();
    guard.type_id = "ai.guardrails".into();
    manifests.push(guard);
    let found = missing_guardrails(&flow, &Catalog::new(manifests)).unwrap();
    assert!(found.problem.contains("safety rules"));
}
