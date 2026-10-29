//! Golden tests: the PRD hero goals (F-A, F-B, F-C) with scripted Architect replies compile into
//! valid, laid-out flows, and a build streams them as events and saves them.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use std::time::Duration;

use il_architect::Architect;
use il_architect::catalog::Catalog;
use il_architect::compile::{CompileError, MAX_REPAIRS, Outcome, compile};
use il_architect::validate::validate;
use il_core::dto::{BuildAnswers, BuildRequest};
use il_core::events::BuildNodeStatus;
use il_core::{BuildEvent, BuildEventKind, NodeManifest};
use il_sdk::services::ArchitectService;
use il_sdk::services::models::ChatRole;
use il_sdk::services::store::Store;
use il_sdk::testkit::{FakeGateway, FakeRunner, FakeServices};
use serde_json::{Value, json};

const FA: &str = "Every Monday, take our three newest Shopify products, make a short promo video for each with a \
    voiceover, cut it for Reels, TikTok, Shorts and LinkedIn with captions, write the post text, and post once I \
    approve in Slack.";
const FB: &str = "Reply to emails sent to support@ using our help center and policies. Send simple answers \
    automatically; ask me in Slack before sending anything about refunds or complaints. Log everything in a sheet.";
const FC: &str = "A support chat agent that answers from these documents and can look up invoices through our API.";

/// The real catalog, as generated from the registered manifests into the web fixtures.
fn catalog() -> Catalog {
    let v: Value = serde_json::from_str(include_str!("../../../web/fixtures/api/nodes.json")).unwrap();
    Catalog::new(serde_json::from_value::<Vec<NodeManifest>>(v["nodes"].clone()).unwrap())
}

fn draft(name: &str) -> Value {
    let text = match name {
        "fa" => include_str!("golden/fa.json"),
        "fb" => include_str!("golden/fb.json"),
        _ => include_str!("golden/fc.json"),
    };
    serde_json::from_str(text).unwrap()
}

fn last_user_message(gw: &FakeGateway) -> String {
    let calls = gw.calls();
    let (_, req) = calls.last().unwrap();
    req.messages.iter().rev().find(|m| m.role == ChatRole::User).unwrap().content.clone()
}

#[tokio::test]
async fn hero_goals_compile_into_valid_flows() {
    let catalog = catalog();
    for (goal, name, nodes, edges) in [(FA, "fa", 18, 18), (FB, "fb", 10, 12), (FC, "fc", 8, 7)] {
        let gw = FakeGateway::default();
        gw.push_json(draft(name));
        let compiled = compile(&gw, &catalog, goal, goal, true).await.unwrap();
        let Outcome::Flow { flow, notes, .. } = compiled.outcome else { panic!("{name}: asked questions") };
        assert!(compiled.repairs.is_empty(), "{name}: needed repairs");
        assert_eq!(validate(&flow, &catalog), vec![], "{name}");
        assert_eq!((flow.nodes.len(), flow.edges.len()), (nodes, edges), "{name}");
        assert!(!notes.is_empty());
        assert_eq!(flow.meta.goal.as_deref(), Some(goal));

        let spots: BTreeSet<(i64, i64)> =
            flow.nodes.iter().map(|n| (n.position.x as i64, n.position.y as i64)).collect();
        assert_eq!(spots.len(), flow.nodes.len(), "{name}: overlapping steps");
        for e in &flow.edges {
            let (from, to) = (flow.node(&e.from.node).unwrap(), flow.node(&e.to.node).unwrap());
            assert!(from.position.x < to.position.x, "{name}: {} is not left of {}", from.id, to.id);
        }

        let (role, req) = gw.calls().remove(0);
        assert_eq!(role, il_sdk::services::models::ModelRole::Architect);
        assert!(req.json_schema.is_some());
        assert!(req.messages[0].content.contains("- trigger.schedule (trigger) \"Schedule\""));
    }
}

#[tokio::test]
async fn repairs_invalid_drafts_with_the_validation_errors() {
    let mut bad = draft("fa");
    bad["nodes"][0]["type"] = json!("trigger.cron");
    bad["nodes"][3]["settings"] = json!({"tone": "upbeat"});
    let gw = FakeGateway::default();
    gw.push_text(format!("Here you go:\n```json\n{bad}\n```"));
    gw.push_json(draft("fa"));

    let compiled = compile(&gw, &catalog(), FA, FA, true).await.unwrap();
    assert!(matches!(compiled.outcome, Outcome::Flow { .. }));
    assert_eq!(compiled.repairs, vec![3]);
    let repair = last_user_message(&gw);
    assert!(repair.contains("nodes.n_schedule: unknown node type `trigger.cron`"), "{repair}");
    assert!(repair.contains("nodes.n_script.settings/instructions: is required"), "{repair}");
    assert!(repair.contains("the flow needs a trigger step"), "{repair}");
}

#[tokio::test]
async fn gives_up_after_the_last_repair() {
    let gw = FakeGateway::default();
    for _ in 0..=MAX_REPAIRS {
        gw.push_text("not json");
    }
    let err = compile(&gw, &catalog(), FA, FA, true).await.unwrap_err();
    assert!(matches!(err, CompileError::Invalid(_)));
    assert_eq!(gw.calls().len(), MAX_REPAIRS + 1);
}

fn request(goal: &str) -> BuildRequest {
    BuildRequest { goal: goal.into(), kind: None, attachments: vec![], hints: vec![] }
}

/// Waits until the build has an event matching `until`, then returns all its events.
async fn events(store: &dyn Store, id: &str, until: fn(&BuildEventKind) -> bool) -> Vec<BuildEvent> {
    for _ in 0..500 {
        let evs = store.build_events(id, 0).await.unwrap();
        if evs.iter().any(|e| until(&e.kind)) {
            return evs;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("build {id} never got there: {:#?}", store.build_events(id, 0).await.unwrap());
}

fn ended(k: &BuildEventKind) -> bool {
    matches!(k, BuildEventKind::Ready { .. } | BuildEventKind::Failed { .. })
}

#[tokio::test]
async fn a03_every_step_card_builds_then_ends_ready() {
    let fakes = FakeServices::default();
    fakes.models.push_json(draft("fa"));
    let architect = Architect::new(fakes.services(), Arc::new(FakeRunner::default()), catalog());
    let id = architect.start_build("ws_1", request(FA)).await.unwrap();
    let evs = events(fakes.store.as_ref(), &id, ended).await;
    let BuildEventKind::Ready { flow_id } = &evs.last().unwrap().kind else { panic!("{evs:#?}") };
    let saved = fakes.store.get_flow(flow_id).await.unwrap().unwrap();
    for node in &saved.nodes {
        let states: Vec<_> = evs
            .iter()
            .filter(|e| e.node_id.as_deref() == Some(node.id.as_str()))
            .filter_map(|e| match &e.kind {
                BuildEventKind::Node { status, .. } => Some(*status),
                _ => None,
            })
            .collect();
        assert_eq!(states.first(), Some(&BuildNodeStatus::Building), "{}", node.id);
        assert!(
            matches!(states.last(), Some(BuildNodeStatus::Ready | BuildNodeStatus::Fixed | BuildNodeStatus::Warning)),
            "{}: {states:?}",
            node.id
        );
    }
    assert!(saved.node("n_tests").is_none(), "the test-suite card is never saved");
}

#[tokio::test]
async fn b10_builds_refuse_to_start_over_the_daily_cap() {
    use il_sdk::services::store::SpendEntry;
    let fakes = FakeServices::default();
    let entry = SpendEntry {
        workspace_id: "ws_1".into(),
        flow_id: None,
        run_id: None,
        build_id: Some("bld_old".into()),
        role: "architect".into(),
        cost_usd: 5.5,
        at: fakes.services().clock.now(),
    };
    fakes.store.record_spend(&entry).await.unwrap();
    let caps = il_architect::BuildCaps { workspace_daily_usd: 5.0, global_daily_usd: 25.0 };
    let architect = Architect::new(fakes.services(), Arc::new(FakeRunner::default()), catalog()).with_caps(caps);
    let id = architect.start_build("ws_1", request(FA)).await.unwrap();
    let evs = events(fakes.store.as_ref(), &id, ended).await;
    let BuildEventKind::Failed { error } = &evs.last().unwrap().kind else { panic!("{evs:#?}") };
    assert!(error.contains("spend cap reached") && error.contains("$5.50 of $5.00"), "{error}");
    assert!(fakes.models.calls().is_empty(), "no model was called");
}

#[tokio::test]
async fn a_build_streams_the_flow_and_saves_it() {
    let fakes = FakeServices::default();
    fakes.models.push_json(draft("fa"));
    let architect = Architect::new(fakes.services(), Arc::new(FakeRunner::default()), catalog());
    let id = architect.start_build("ws_1", request(FA)).await.unwrap();
    let evs = events(fakes.store.as_ref(), &id, ended).await;

    let BuildEventKind::Ready { flow_id } = &evs.last().unwrap().kind else { panic!("{evs:#?}") };
    let saved = fakes.store.get_flow(flow_id).await.unwrap().unwrap();
    assert_eq!(saved.workspace_id, "ws_1");
    assert!(evs.windows(2).all(|w| w[1].seq == w[0].seq + 1));

    // Replaying the patches onto an empty flow gives the saved flow's groups, nodes and edges.
    let mut shown = json!({"nodes": [], "edges": [], "groups": []});
    for e in &evs {
        if let BuildEventKind::FlowPatch { json_patch } = &e.kind {
            let op = &json_patch[0];
            let path = op["path"].as_str().unwrap();
            match path.strip_suffix("/-") {
                Some(list) => shown[&list[1..]].as_array_mut().unwrap().push(op["value"].clone()),
                None => shown[&path[1..]] = op["value"].clone(),
            }
        }
    }
    let saved_json = serde_json::to_value(&saved).unwrap();
    for key in ["groups", "nodes", "edges"] {
        let ids = |v: &Value| v[key].as_array().unwrap().iter().map(|x| x.to_string()).collect::<BTreeSet<_>>();
        assert_eq!(ids(&shown), ids(&saved_json), "{key}");
    }

    let stages: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.kind {
            BuildEventKind::Stage { stage, status } => Some(format!("{stage}:{status:?}")),
            _ => None,
        })
        .collect();
    assert_eq!(
        stages,
        ["understand:Active", "understand:Done", "assemble:Active", "assemble:Done", "test:Active", "test:Done"]
    );
    assert!(evs.iter().any(|e| matches!(e.kind, BuildEventKind::TestResult { passed: 1, failed: 0 })));
    let apps: Vec<_> = evs
        .iter()
        .filter_map(|e| match &e.kind {
            BuildEventKind::ConnectionNeeded { app } => Some((app.as_str(), e.node_id.as_deref().unwrap_or_default())),
            _ => None,
        })
        .collect();
    let unique: BTreeMap<_, _> = apps.iter().copied().collect();
    assert_eq!(unique.len(), apps.len(), "an app was flagged twice");
    assert_eq!(unique.get("shopify"), Some(&"n_products"));
    assert_eq!(unique.get("slack"), Some(&"n_approve"));
}

#[tokio::test]
async fn asks_questions_then_builds_with_the_answers() {
    let fakes = FakeServices::default();
    fakes.models.push_json(json!({
        "name": "Support inbox replies", "nodes": [], "edges": [],
        "questions": [{"id": "q_channel", "text": "Which Slack channel should approvals go to?", "options": ["#support", "#ops"]}]
    }));
    fakes.models.push_json(draft("fb"));
    let architect = Architect::new(fakes.services(), Arc::new(FakeRunner::default()), catalog());
    let id = architect.start_build("ws_1", request(FB)).await.unwrap();

    events(fakes.store.as_ref(), &id, |k| matches!(k, BuildEventKind::Question { .. })).await;
    let answers = BuildAnswers { answers: [("q_channel".to_string(), "#support".to_string())].into() };
    architect.answer(&id, answers).await.unwrap();
    let evs = events(fakes.store.as_ref(), &id, ended).await;

    assert!(matches!(evs.last().unwrap().kind, BuildEventKind::Ready { .. }), "{evs:#?}");
    assert!(last_user_message(&fakes.models).contains("Which Slack channel should approvals go to? #support"));
    assert!(architect.answer(&id, BuildAnswers { answers: BTreeMap::new() }).await.is_err());
}

#[tokio::test]
async fn a_build_that_cannot_be_repaired_fails_with_the_reason() {
    let fakes = FakeServices::default();
    for _ in 0..=MAX_REPAIRS {
        fakes
            .models
            .push_json(json!({"name": "x", "nodes": [{"id": "n_a", "type": "nope", "name": "A"}], "edges": []}));
    }
    let architect = Architect::new(fakes.services(), Arc::new(FakeRunner::default()), catalog());
    let id = architect.start_build("ws_1", request("Do something")).await.unwrap();
    let evs = events(fakes.store.as_ref(), &id, ended).await;
    let BuildEventKind::Failed { error } = &evs.last().unwrap().kind else { panic!("{evs:#?}") };
    assert!(error.contains("unknown node type `nope`"), "{error}");
    assert!(
        evs.iter()
            .any(|e| matches!(&e.kind, BuildEventKind::Stage { status, .. } if format!("{status:?}") == "Failed"))
    );
    assert!(fakes.store.list_flows("ws_1").await.unwrap().is_empty());
}
