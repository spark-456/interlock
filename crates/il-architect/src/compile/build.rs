//! A build: compile in the background and stream every stage as build events, so the canvas fills
//! in from real progress (AR-7). Nodes and edges arrive one `build.flow_patch` at a time.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use il_core::dto::{AskDiff, AttachmentKind, AttachmentRole, BuildAnswers, BuildRequest};
use il_core::events::{BuildQuestion, StageStatus};
use il_core::{BuildEvent, BuildEventKind, Flow};
use il_sdk::Services;
use il_sdk::services::architect::{ArchitectError, ArchitectService};
use serde_json::json;
use tokio::sync::oneshot;

use super::{Outcome, compile};
use crate::catalog::Catalog;

/// How long a build waits for answers to its questions.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// The architect service: builds run in the background and persist their events to the store.
#[derive(Clone)]
pub struct Architect {
    inner: Arc<Inner>,
}

struct Inner {
    services: Services,
    catalog: Catalog,
    /// Builds waiting for answers, by build id.
    waiting: Mutex<HashMap<String, oneshot::Sender<BuildAnswers>>>,
}

impl Architect {
    /// An architect that may use the node types in `catalog`.
    pub fn new(services: Services, catalog: Catalog) -> Self {
        Self { inner: Arc::new(Inner { services, catalog, waiting: Mutex::default() }) }
    }
}

#[async_trait]
impl ArchitectService for Architect {
    async fn start_build(&self, workspace_id: &str, req: BuildRequest) -> Result<String, ArchitectError> {
        let id = il_sdk::id("bld");
        let (inner, build_id, ws) = (self.inner.clone(), id.clone(), workspace_id.to_string());
        tokio::spawn(async move { inner.run(build_id, ws, req).await });
        Ok(id)
    }

    async fn answer(&self, build_id: &str, answers: BuildAnswers) -> Result<(), ArchitectError> {
        let tx = self.inner.waiting.lock().ok().and_then(|mut w| w.remove(build_id));
        tx.ok_or_else(|| ArchitectError::NotFound(build_id.into()))?
            .send(answers)
            .map_err(|_| ArchitectError::Failed("the build stopped waiting for answers".into()))
    }

    async fn ask(&self, _flow: &il_core::Flow, _message: &str) -> Result<AskDiff, ArchitectError> {
        Err(ArchitectError::Failed("Ask Interlock arrives with task A02".into()))
    }
}

/// Appends events for one build, tracking the sequence number, running cost and current stage.
struct Events<'a> {
    services: &'a Services,
    build_id: String,
    seq: u64,
    cost_usd: f64,
    stage: Option<&'static str>,
}

impl Events<'_> {
    async fn emit(&mut self, node_id: Option<&str>, kind: BuildEventKind) {
        self.seq += 1;
        let ev = BuildEvent {
            seq: self.seq,
            ts: self.services.clock.now(),
            build_id: self.build_id.clone(),
            node_id: node_id.map(String::from),
            kind,
        };
        if let Err(e) = self.services.store.append_build_event(&ev).await {
            tracing::warn!(build_id = %self.build_id, error = %e, "could not store a build event");
        }
    }

    async fn log(&mut self, text: impl Into<String>) {
        self.emit(None, BuildEventKind::Log { text: text.into() }).await;
    }

    /// Closes the current stage (if any) and opens `stage`.
    async fn stage(&mut self, stage: &'static str) {
        self.end_stage(StageStatus::Done).await;
        self.stage = Some(stage);
        self.emit(None, BuildEventKind::Stage { stage: stage.into(), status: StageStatus::Active }).await;
    }

    async fn end_stage(&mut self, status: StageStatus) {
        if let Some(stage) = self.stage.take() {
            self.emit(None, BuildEventKind::Stage { stage: stage.into(), status }).await;
        }
    }

    async fn cost(&mut self, usd: f64) {
        self.cost_usd += usd;
        let cost_usd = self.cost_usd;
        self.emit(None, BuildEventKind::Cost { cost_usd }).await;
    }
}

impl Inner {
    async fn run(self: Arc<Self>, build_id: String, ws: String, req: BuildRequest) {
        let mut ev = Events { services: &self.services, build_id, seq: 0, cost_usd: 0.0, stage: None };
        match self.build(&mut ev, &ws, &req).await {
            Ok(flow_id) => {
                ev.end_stage(StageStatus::Done).await;
                ev.emit(None, BuildEventKind::Ready { flow_id }).await;
            }
            Err(error) => {
                tracing::warn!(build_id = %ev.build_id, %error, "build failed");
                ev.end_stage(StageStatus::Failed).await;
                ev.emit(None, BuildEventKind::Failed { error }).await;
            }
        }
    }

    async fn build(&self, ev: &mut Events<'_>, ws: &str, req: &BuildRequest) -> Result<String, String> {
        let models = self.services.models.as_ref();
        ev.stage("understand").await;
        ev.log("Reading your request.").await;
        let mut brief = self.brief(ev, req).await;
        let mut compiled = compile(models, &self.catalog, &req.goal, &brief, true).await.map_err(|e| e.to_string())?;
        ev.cost(compiled.cost_usd).await;
        if let Outcome::Questions(questions) = &compiled.outcome {
            let answers = self.ask_questions(ev, questions.clone()).await?;
            brief.push_str("\n\nAnswers to your questions (do not ask again):");
            for q in questions {
                let a = answers.answers.get(&q.id).map_or("no preference", String::as_str);
                let _ = write!(brief, "\n- {} {a}", q.text);
            }
            compiled = compile(models, &self.catalog, &req.goal, &brief, false).await.map_err(|e| e.to_string())?;
            ev.cost(compiled.cost_usd).await;
        }
        for fixed in &compiled.repairs {
            ev.log(format!("Checked the draft and fixed {fixed} problem{}.", if *fixed == 1 { "" } else { "s" })).await;
        }
        let Outcome::Flow { mut flow, notes } = compiled.outcome else {
            return Err("the architect asked questions again after they were answered".into());
        };
        for note in notes {
            ev.log(note).await;
        }
        ev.stage("assemble").await;
        flow.workspace_id = ws.into();
        if let Some(kind) = req.kind {
            flow.kind = kind;
        }
        self.stream(ev, &flow).await;
        self.services.store.put_flow(&flow).await.map_err(|e| format!("could not save the flow: {e}"))?;
        Ok(flow.id)
    }

    /// The goal plus attachments and hints; voice memos marked Instructions are transcribed.
    async fn brief(&self, ev: &mut Events<'_>, req: &BuildRequest) -> String {
        let mut brief = format!("Goal: {}", req.goal.trim());
        if let Some(kind) = req.kind {
            let _ = write!(brief, "\nBuild it as: {}", json!(kind).as_str().unwrap_or_default());
        }
        if !req.hints.is_empty() {
            let _ = write!(brief, "\nApps and APIs to use: {}", req.hints.join(", "));
        }
        for a in &req.attachments {
            let role = match a.role {
                AttachmentRole::Knowledge => "knowledge to answer from",
                AttachmentRole::Instructions => "instructions to follow",
            };
            let _ = write!(brief, "\nAttachment ({role}): {}", a.url.as_deref().unwrap_or(&a.name));
            if a.kind == AttachmentKind::Audio
                && a.role == AttachmentRole::Instructions
                && let Some(text) = self.transcribe(a.file_id.as_deref()).await
            {
                ev.log(format!("Transcribed {}.", a.name)).await;
                let _ = write!(brief, "\n  Transcript: {text}");
            }
        }
        brief
    }

    async fn transcribe(&self, file_id: Option<&str>) -> Option<String> {
        let (file, bytes) = self.services.files.get(file_id?).await.ok()?;
        match self.services.models.transcribe(&file, &bytes).await {
            Ok(t) => Some(t.text),
            Err(e) => {
                tracing::warn!(file_id = %file.id, error = %e, "could not transcribe an attachment");
                None
            }
        }
    }

    async fn ask_questions(&self, ev: &mut Events<'_>, questions: Vec<BuildQuestion>) -> Result<BuildAnswers, String> {
        let (tx, rx) = oneshot::channel();
        if let Ok(mut w) = self.waiting.lock() {
            w.insert(ev.build_id.clone(), tx);
        }
        ev.emit(None, BuildEventKind::Question { questions }).await;
        let answers = tokio::time::timeout(ANSWER_TIMEOUT, rx).await;
        if let Ok(mut w) = self.waiting.lock() {
            w.remove(&ev.build_id);
        }
        match answers {
            Ok(Ok(a)) => Ok(a),
            _ => Err("no answers to the questions, so the build stopped".into()),
        }
    }

    /// Groups first, then each step left to right with the edges it completes, flagging every app
    /// that still needs a connection once (AR-5).
    async fn stream(&self, ev: &mut Events<'_>, flow: &Flow) {
        let patch = |path: &str, value: serde_json::Value| BuildEventKind::FlowPatch {
            json_patch: json!([{"op": "add", "path": path, "value": value}]),
        };
        ev.emit(None, patch("/groups", json!(flow.groups))).await;
        let mut order: Vec<_> = flow.nodes.iter().collect();
        order.sort_by(|a, b| a.position.x.total_cmp(&b.position.x).then(a.position.y.total_cmp(&b.position.y)));
        let mut placed = BTreeSet::new();
        let mut apps = BTreeSet::new();
        for n in order {
            ev.emit(Some(&n.id), patch("/nodes/-", json!(n))).await;
            placed.insert(n.id.as_str());
            for e in flow.edges.iter().filter(|e| e.to.node == n.id || e.from.node == n.id) {
                if placed.contains(e.from.node.as_str()) && placed.contains(e.to.node.as_str()) {
                    ev.emit(None, patch("/edges/-", json!(e))).await;
                }
            }
            let specs = self.catalog.get(&n.node_type).map(|m| m.credentials.as_slice()).unwrap_or_default();
            for c in specs.iter().filter(|c| !n.credentials.contains_key(&c.key)) {
                if apps.insert(c.app.clone()) {
                    ev.emit(Some(&n.id), BuildEventKind::ConnectionNeeded { app: c.app.clone() }).await;
                }
            }
        }
    }
}
