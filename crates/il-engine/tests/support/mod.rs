//! Fake nodes and helpers for engine tests.
#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use il_core::{Category, Decision, Edge, FileRef, Flow, FlowKind, Item, Node, NodeManifest, PortRef, Position};
use il_engine::{Engine, EngineConfig, NodeLookup};
use il_sdk::ctx::{ApprovalSpec, NodeCtx};
use il_sdk::services::{Services, SystemClock};
use il_sdk::testkit::FakeServices;
use il_sdk::{NodeError, NodeExecutor, NodeInput, NodeOutput};
use serde_json::{Value, json};

/// A fake node: manifest + closure-free behaviour selected by type id.
pub struct Fake {
    manifest: NodeManifest,
    pub concurrent: Arc<AtomicUsize>,
    pub max_concurrent: Arc<AtomicUsize>,
}

pub fn m(id: &str) -> NodeManifest {
    NodeManifest::new(id, Category::Data, id, "test node")
}

impl Fake {
    fn new(manifest: NodeManifest) -> Arc<Self> {
        Arc::new(Self { manifest, concurrent: Arc::default(), max_concurrent: Arc::default() })
    }
}

#[async_trait]
impl NodeExecutor for Fake {
    fn manifest(&self) -> &NodeManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let s = ctx.settings.clone();
        let items = input.main().to_vec();
        match self.manifest.type_id.as_str() {
            "test.trigger" => Ok(NodeOutput::out(items)),
            "test.double" => Ok(NodeOutput::out(
                items
                    .into_iter()
                    .map(|mut i| {
                        i.json["n"] = json!(i.json["n"].as_i64().unwrap_or(0) * 2);
                        i
                    })
                    .collect(),
            )),
            "test.text" => Ok(NodeOutput::out(
                items
                    .into_iter()
                    .map(|mut i| {
                        i.json["text"] = s["text"].clone();
                        i
                    })
                    .collect(),
            )),
            "test.field" => Ok(NodeOutput::out(
                items
                    .into_iter()
                    .map(|mut i| {
                        i.json[s["field"].as_str().unwrap_or("field")] = s["value"].clone();
                        if let Some(key) = s["fileKey"].as_str() {
                            i.files.insert(
                                key.into(),
                                FileRef {
                                    id: key.into(),
                                    mime: "text/plain".into(),
                                    bytes: 1,
                                    name: format!("{key}.txt"),
                                    preview_url: None,
                                },
                            );
                        }
                        i
                    })
                    .collect(),
            )),
            "test.split" => Ok(NodeOutput::out(
                items
                    .into_iter()
                    .flat_map(|item| {
                        (0..2).map(move |part| {
                            let mut child = item.clone();
                            child.json["part"] = json!(part);
                            child
                        })
                    })
                    .collect(),
            )),
            "test.kb" => Ok(NodeOutput::port("kb", vec![Item::json(json!({ "knowledgeBase": "kb_real" }))])),
            "test.bases" => Ok(NodeOutput::out(
                items
                    .into_iter()
                    .map(|mut i| {
                        i.json["bases"] = s["knowledgeBases"].clone();
                        i
                    })
                    .collect(),
            )),
            "test.branch" => {
                let item = items.into_iter().next().unwrap_or_default();
                let port = if item.json["n"].as_i64().unwrap_or(0) % 2 == 0 { "even" } else { "odd" };
                Ok(NodeOutput::port(port, vec![item]))
            }
            "test.collect" => {
                let all: Vec<&Item> = input.ports.values().flatten().collect();
                let sum: i64 = all.iter().map(|i| i.json["n"].as_i64().unwrap_or(0)).sum();
                Ok(NodeOutput::out(vec![Item::json(json!({ "sum": sum, "count": all.len() }))]))
            }
            "test.flaky" => {
                let failures = s["failures"].as_u64().unwrap_or(0) as u32;
                if ctx.run.attempt <= failures {
                    Err(NodeError::Retryable(format!("flaky attempt {}", ctx.run.attempt)))
                } else {
                    Ok(NodeOutput::out(items))
                }
            }
            "test.fail" => Err(NodeError::Failed("boom".into())),
            "test.slow" => {
                let now = self.concurrent.fetch_add(1, Ordering::SeqCst) + 1;
                self.max_concurrent.fetch_max(now, Ordering::SeqCst);
                let ms = s["ms"].as_u64().unwrap_or(50);
                tokio::time::sleep(Duration::from_millis(ms)).await;
                self.concurrent.fetch_sub(1, Ordering::SeqCst);
                Ok(NodeOutput::out(items))
            }
            "test.cost" => {
                let mut out = NodeOutput::out(items);
                out.cost_usd = s["usd"].as_f64().unwrap_or(0.0);
                Ok(out)
            }
            "test.approve" => match &ctx.resume {
                None => {
                    let spec = ApprovalSpec {
                        title: "Approve?".into(),
                        message: "Please decide".into(),
                        payload: items.first().map(|i| i.json.clone()).unwrap_or(Value::Null),
                        channels: vec!["inapp".into(), "slack".into()],
                        target: Some("#marketing".into()),
                        timeout_hours: s["timeoutHours"].as_f64(),
                        on_timeout: match s["onTimeout"].as_str() {
                            Some("approve") => il_core::ApprovalTimeout::Approve,
                            Some("reject") => il_core::ApprovalTimeout::Reject,
                            _ => il_core::ApprovalTimeout::KeepWaiting,
                        },
                        credential_id: None,
                    };
                    Ok(NodeOutput::waiting(ctx.request_approval(spec).await?))
                }
                Some(d) => {
                    let mut item = items.into_iter().next().unwrap_or_default();
                    if let Some(p) = &d.payload {
                        item.json = p.clone();
                    }
                    let port = if d.decision == Decision::Approve { "approved" } else { "rejected" };
                    Ok(NodeOutput::port(port, vec![item]))
                }
            },
            "output.respond" => Ok(NodeOutput::out(vec![Item::json(json!({ "answer": s["body"] }))])),
            other => Err(NodeError::NotImplemented(other.into())),
        }
    }
}

/// All fake node types; `slow` is returned so tests can read its concurrency counters.
pub fn lookup() -> (NodeLookup, Arc<Fake>) {
    let slow = Fake::new(m("test.slow"));
    let mut l = NodeLookup::default()
        .with(Fake::new(m("test.trigger").trigger()))
        .with(Fake::new(m("test.double")))
        .with(Fake::new(m("test.text")))
        .with(Fake::new(m("test.field")))
        .with(Fake::new(m("test.split")))
        .with(Fake::new(m("test.kb").inputs(&[]).outputs(&["kb"])))
        .with(Fake::new(m("test.bases")))
        .with(Fake::new(m("test.branch").outputs(&["even", "odd"])))
        .with(Fake::new(m("test.collect").inputs(&["a", "b"]).batch()))
        .with(Fake::new(m("test.flaky")))
        .with(Fake::new(m("test.fail")))
        .with(Fake::new(m("test.cost")))
        .with(Fake::new(m("test.approve").outputs(&["approved", "rejected"])))
        .with(Fake::new(m("output.respond").outputs(&[])));
    l = l.with(slow.clone());
    (l, slow)
}

/// Fake services with a real clock (retries and timeouts sleep for real, briefly).
pub fn services() -> (FakeServices, Services) {
    let fakes = FakeServices::default();
    let mut svc = fakes.services();
    svc.clock = Arc::new(SystemClock);
    (fakes, svc)
}

pub fn config() -> EngineConfig {
    EngineConfig { max_parallel: 4, retries: 2, retry_backoff_ms: 1, node_timeout_ms: 2_000, ..Default::default() }
}

pub fn engine_with(svc: Services, config: EngineConfig) -> (Engine, Arc<Fake>) {
    let (lookup, slow) = lookup();
    (Engine::new(svc, lookup, config), slow)
}

/// Builds a flow from `(id, type, settings)` and `(from, from_port, to, to_port)`.
pub fn flow(nodes: &[(&str, &str, Value)], edges: &[(&str, &str, &str, &str)]) -> Flow {
    Flow {
        id: "flw_t".into(),
        workspace_id: "ws_t".into(),
        version: 1,
        name: "Test".into(),
        kind: FlowKind::Automation,
        nodes: nodes
            .iter()
            .map(|(id, ty, settings)| Node {
                id: (*id).into(),
                node_type: (*ty).into(),
                type_version: 1,
                name: (*id).into(),
                position: Position::default(),
                group: None,
                settings: settings.clone(),
                credentials: Default::default(),
                disabled: false,
                notes: None,
            })
            .collect(),
        edges: edges
            .iter()
            .enumerate()
            .map(|(i, (a, ap, b, bp))| Edge {
                id: format!("e{i}"),
                from: PortRef { node: (*a).into(), port: (*ap).into() },
                to: PortRef { node: (*b).into(), port: (*bp).into() },
            })
            .collect(),
        groups: vec![],
        settings: Default::default(),
        meta: Default::default(),
    }
}

pub fn items(ns: &[i64]) -> Vec<Item> {
    ns.iter().map(|n| Item::json(json!({ "n": n }))).collect()
}
