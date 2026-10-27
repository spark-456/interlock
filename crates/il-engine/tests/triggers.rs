//! Trigger supervisor: activate / deactivate lifetimes, all-or-nothing starts, restore, and the
//! chat/API/MCP invoke entry point.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use il_core::{Category, Edge, Flow, FlowKind, Item, Node, NodeManifest, PortRef};
use il_engine::NodeLookup;
use il_engine::triggers::{TriggerError, TriggerSupervisor};
use il_sdk::ctx::NodeCtx;
use il_sdk::services::Store;
use il_sdk::testkit::{FakeRunner, FakeServices};
use il_sdk::{NodeError, NodeExecutor, NodeInput, NodeOutput, TriggerCtx, TriggerHandle, TriggerNode};
use serde_json::json;
use tokio_util::sync::CancellationToken;

/// A trigger that counts starts and live handles; `fail: true` in settings refuses to start.
struct Counting {
    manifest: NodeManifest,
    started: Arc<AtomicUsize>,
    live: Arc<AtomicUsize>,
}

#[async_trait]
impl NodeExecutor for Counting {
    fn manifest(&self) -> &NodeManifest {
        &self.manifest
    }
    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }
    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for Counting {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        if ctx.settings["fail"] == true {
            return Err(NodeError::Failed("refused".into()));
        }
        self.started.fetch_add(1, Ordering::SeqCst);
        self.live.fetch_add(1, Ordering::SeqCst);
        let live = self.live.clone();
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let task = tokio::spawn(async move {
            token.cancelled().await;
            live.fetch_sub(1, Ordering::SeqCst);
        });
        Ok(TriggerHandle::new(cancel, Some(task)))
    }
}

fn node(id: &str, ty: &str, settings: serde_json::Value) -> Node {
    Node {
        id: id.into(),
        node_type: ty.into(),
        type_version: 1,
        name: id.into(),
        position: Default::default(),
        group: None,
        settings,
        credentials: Default::default(),
        disabled: false,
        notes: None,
    }
}

fn flow(nodes: Vec<Node>) -> Flow {
    Flow {
        id: "flw".into(),
        workspace_id: "ws".into(),
        version: 1,
        name: "F".into(),
        kind: FlowKind::App,
        edges: vec![Edge {
            id: "e".into(),
            from: PortRef { node: nodes[0].id.clone(), port: "out".into() },
            to: PortRef { node: "n_work".into(), port: "in".into() },
        }],
        nodes,
        groups: vec![],
        settings: Default::default(),
        meta: Default::default(),
    }
}

struct Setup {
    sup: TriggerSupervisor,
    fakes: FakeServices,
    runner: Arc<FakeRunner>,
    started: Arc<AtomicUsize>,
    live: Arc<AtomicUsize>,
}

fn setup() -> Setup {
    let fakes = FakeServices::default();
    let runner = Arc::new(FakeRunner::default());
    let (started, live) = (Arc::new(AtomicUsize::new(0)), Arc::new(AtomicUsize::new(0)));
    let counting = Counting {
        manifest: NodeManifest::new("test.counting", Category::Trigger, "Counting", "test").trigger(),
        started: started.clone(),
        live: live.clone(),
    };
    let chat = Counting {
        manifest: NodeManifest::new("trigger.chat", Category::Trigger, "Chat", "test").trigger(),
        started: Arc::default(),
        live: Arc::default(),
    };
    let lookup = NodeLookup::default().with(Arc::new(counting)).with(Arc::new(chat));
    let sup = TriggerSupervisor::new(fakes.services(), runner.clone(), lookup);
    Setup { sup, fakes, runner, started, live }
}

#[tokio::test]
async fn activate_starts_triggers_and_deactivate_stops_them() {
    let s = setup();
    let f = flow(vec![
        node("n_a", "test.counting", json!({})),
        node("n_b", "test.counting", json!({})),
        node("n_work", "test.unrelated", json!({})),
    ]);
    s.fakes.store.put_flow(&f).await.unwrap();
    assert_eq!(s.sup.activate("flw").await.unwrap(), 2);
    assert!(s.fakes.store.is_flow_active("flw").await.unwrap());
    assert_eq!(s.live.load(Ordering::SeqCst), 2);
    // Re-activating restarts instead of doubling.
    s.sup.activate("flw").await.unwrap();
    assert_eq!(s.live.load(Ordering::SeqCst), 2);
    assert_eq!(s.started.load(Ordering::SeqCst), 4);
    s.sup.deactivate("flw").await.unwrap();
    assert_eq!(s.live.load(Ordering::SeqCst), 0);
    assert!(!s.fakes.store.is_flow_active("flw").await.unwrap());
    assert!(!s.sup.is_running("flw").await);
}

#[tokio::test]
async fn a_failing_trigger_rolls_back_the_activation() {
    let s = setup();
    let f = flow(vec![
        node("n_ok", "test.counting", json!({})),
        node("n_bad", "test.counting", json!({ "fail": true })),
        node("n_work", "test.unrelated", json!({})),
    ]);
    s.fakes.store.put_flow(&f).await.unwrap();
    let err = s.sup.activate("flw").await.unwrap_err();
    assert!(matches!(err, TriggerError::Start { ref node, .. } if node == "n_bad"));
    assert_eq!(s.live.load(Ordering::SeqCst), 0, "already-started triggers are stopped");
    assert!(!s.fakes.store.is_flow_active("flw").await.unwrap());
}

#[tokio::test]
async fn restore_restarts_active_flows() {
    let s = setup();
    let f = flow(vec![node("n_a", "test.counting", json!({})), node("n_work", "test.unrelated", json!({}))]);
    s.fakes.store.put_flow(&f).await.unwrap();
    s.fakes.store.set_flow_active("flw", true).await.unwrap();
    assert_eq!(s.sup.restore(&["flw".into(), "flw_missing".into()]).await, 1);
    assert_eq!(s.live.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn invoke_runs_from_the_matching_trigger() {
    let s = setup();
    let f = flow(vec![node("n_chat", "trigger.chat", json!({})), node("n_work", "test.unrelated", json!({}))]);
    s.fakes.store.put_flow(&f).await.unwrap();
    let out = s.sup.invoke("flw", "trigger.chat", vec![Item::json(json!({ "message": "hi" }))]).await.unwrap();
    assert_eq!(out.response[0].json["message"], "hi");
    let started = s.runner.started();
    assert_eq!(started[0].trigger_node.as_deref(), Some("n_chat"));
    assert_eq!(started[0].trigger, "trigger.chat");
    assert!(matches!(s.sup.invoke("flw", "trigger.mcp_call", vec![]).await, Err(TriggerError::NoTrigger(_))));
}
