//! A build: compile in the background and stream every stage as build events, so the canvas fills
//! in from real progress (AR-7). Nodes and edges arrive one `build.flow_patch` at a time.

use std::collections::{BTreeSet, HashMap};
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use il_core::dto::{AskDiff, AttachmentKind, AttachmentRole, BuildAnswers, BuildRequest};
use il_core::events::{BuildNodeStatus, BuildQuestion, StageStatus};
use il_core::{BuildEvent, BuildEventKind, Flow, FlowKind, Json};
use il_sdk::Services;
use il_sdk::services::architect::{ArchitectError, ArchitectService};
use il_sdk::services::metering::{ModelCallContext, with_model_context};
use il_sdk::services::runner::FlowRunner;
use il_sdk::services::store::{SpendEntry, SpendScope};
use serde_json::json;
use tokio::sync::oneshot;

use super::{Compiled, Draft, Outcome, assemble, compile, progress, revise, to_draft};
use crate::appkit::{self, Case};
use crate::ask::{changed, diff};
use crate::catalog::Catalog;
use crate::testfix::{Failure, MAX_FIX_ROUNDS, fix_request, smoke};

/// How long a build waits for answers to its questions.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(30 * 60);

fn words(text: &str) -> BTreeSet<String> {
    text.split(|c: char| !c.is_alphanumeric()).filter(|word| word.len() >= 3).map(str::to_lowercase).collect()
}

/// The architect service: builds run in the background and persist their events to the store.
#[derive(Clone)]
pub struct Architect {
    inner: Arc<Inner>,
}

struct Inner {
    services: Services,
    runner: Arc<dyn FlowRunner>,
    catalog: Catalog,
    fake_templates: bool,
    /// Daily caps a build must be under to start; none in tests.
    caps: Option<BuildCaps>,
    /// Builds waiting for answers, by build id.
    waiting: Mutex<HashMap<String, oneshot::Sender<BuildAnswers>>>,
}

impl Architect {
    /// An architect that may use the node types in `catalog` and test-runs flows with `runner`.
    pub fn new(services: Services, runner: Arc<dyn FlowRunner>, catalog: Catalog) -> Self {
        Self::new_with_fake_templates(services, runner, catalog, false)
    }

    /// Uses the workspace's seeded templates for offline builds when `fake_templates` is true.
    pub fn new_with_fake_templates(
        services: Services,
        runner: Arc<dyn FlowRunner>,
        catalog: Catalog,
        fake_templates: bool,
    ) -> Self {
        let inner = Inner { services, runner, catalog, fake_templates, caps: None, waiting: Mutex::default() };
        Self { inner: Arc::new(inner) }
    }

    /// Refuses to start builds once the workspace's or everyone's spend today reaches `caps`.
    pub fn with_caps(mut self, caps: BuildCaps) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.inner) {
            inner.caps = Some(caps);
        }
        self
    }
}

/// The `[caps]` limits builds count toward (USD per day).
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(default)]
pub struct BuildCaps {
    pub workspace_daily_usd: f64,
    pub global_daily_usd: f64,
}

impl Default for BuildCaps {
    fn default() -> Self {
        Self { workspace_daily_usd: 5.0, global_daily_usd: 25.0 }
    }
}

#[async_trait]
impl ArchitectService for Architect {
    async fn start_build(&self, workspace_id: &str, req: BuildRequest) -> Result<String, ArchitectError> {
        let id = il_sdk::id("bld");
        let (inner, build_id, ws) = (self.inner.clone(), id.clone(), workspace_id.to_string());
        tokio::spawn(async move {
            let context = ModelCallContext {
                workspace_id: ws.clone(),
                flow_id: None,
                run_id: None,
                build_id: Some(build_id.clone()),
            };
            with_model_context(context, inner.run(build_id, ws, req)).await;
        });
        Ok(id)
    }

    async fn answer(&self, build_id: &str, answers: BuildAnswers) -> Result<(), ArchitectError> {
        let tx = self.inner.waiting.lock().ok().and_then(|mut w| w.remove(build_id));
        tx.ok_or_else(|| ArchitectError::NotFound(build_id.into()))?
            .send(answers)
            .map_err(|_| ArchitectError::Failed("the build stopped waiting for answers".into()))
    }

    async fn ask(&self, flow: &Flow, message: &str) -> Result<AskDiff, ArchitectError> {
        let models = self.inner.services.models.as_ref();
        let context = ModelCallContext {
            workspace_id: flow.workspace_id.clone(),
            flow_id: Some(flow.id.clone()),
            run_id: None,
            build_id: None,
        };
        let (diff, _) = with_model_context(context, crate::ask::ask(models, &self.inner.catalog, flow, message))
            .await
            .map_err(|e| ArchitectError::Failed(e.to_string()))?;
        Ok(diff)
    }
}

/// Appends events for one build, tracking the sequence number, running cost and current stage.
struct Events<'a> {
    services: &'a Services,
    build_id: String,
    workspace_id: String,
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
        if usd > 0.0 {
            let entry = SpendEntry {
                workspace_id: self.workspace_id.clone(),
                flow_id: None,
                run_id: None,
                build_id: Some(self.build_id.clone()),
                role: "architect".into(),
                cost_usd: usd,
                at: self.services.clock.now(),
            };
            if let Err(e) = self.services.store.record_spend(&entry).await {
                tracing::warn!(build_id = %self.build_id, error = %e, "could not record build spend");
            }
        }
        self.cost_usd += usd;
        let cost_usd = self.cost_usd;
        self.emit(None, BuildEventKind::Cost { cost_usd }).await;
    }
}

impl Inner {
    async fn run(self: Arc<Self>, build_id: String, ws: String, req: BuildRequest) {
        let mut ev =
            Events { services: &self.services, build_id, workspace_id: ws.clone(), seq: 0, cost_usd: 0.0, stage: None };
        let outcome = match self.over_cap(&ws).await {
            Some(error) => Err(error),
            None => self.build(&mut ev, &ws, &req).await,
        };
        match outcome {
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

    /// Why a build can't start: today's workspace or global spend has reached its cap.
    async fn over_cap(&self, ws: &str) -> Option<String> {
        let caps = self.caps?;
        let now = self.services.clock.now();
        let day = now.date_naive().and_hms_opt(0, 0, 0).map_or(now, |d| d.and_utc());
        let checks = [
            (SpendScope::WorkspaceSince(ws.into(), day), caps.workspace_daily_usd, "This workspace"),
            (SpendScope::GlobalSince(day), caps.global_daily_usd, "Interlock"),
        ];
        for (scope, limit, who) in checks {
            let spent = self.services.store.spend(&scope).await.unwrap_or(0.0);
            if spent >= limit {
                return Some(format!(
                    "spend cap reached: {who} spent ${spent:.2} of ${limit:.2} today, so builds start again tomorrow"
                ));
            }
        }
        None
    }

    async fn build(&self, ev: &mut Events<'_>, ws: &str, req: &BuildRequest) -> Result<String, String> {
        let models = self.services.models.as_ref();
        ev.stage("understand").await;
        ev.log("Reading your request.").await;
        let instructions = super::attach::instructions(&self.services, &req.attachments).await;
        let mut brief = self.brief(ev, req, &instructions).await;
        ev.log("Planning the steps. This can take a minute.").await;
        let mut compiled = if self.fake_templates {
            self.compile_template(ws, &req.goal).await?
        } else {
            compile(models, &self.catalog, &req.goal, &brief, true).await.map_err(|e| e.to_string())?
        };
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
        let Outcome::Flow { mut flow, notes, sample } = compiled.outcome else {
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
        for line in super::attach::apply(&mut flow, &req.attachments, &instructions) {
            ev.log(line).await;
        }
        self.stream(ev, &flow).await;
        self.services.store.put_flow(&flow).await.map_err(|e| format!("could not save the flow: {e}"))?;
        let flow_id = flow.id.clone();
        ev.stage("test").await;
        self.test_and_fix(ev, *flow, &sample, &brief).await;
        Ok(flow_id)
    }

    async fn compile_template(&self, ws: &str, goal: &str) -> Result<Compiled, String> {
        let goal_words = words(goal);
        let flows = self.services.store.list_flows(ws).await.map_err(|e| e.to_string())?;
        let template = flows
            .into_iter()
            .filter(|flow| flow.meta.created_by.as_deref() == Some("template"))
            .max_by(|a, b| {
                let score = |flow: &Flow| {
                    goal_words
                        .intersection(&words(&format!("{} {}", flow.name, flow.meta.goal.as_deref().unwrap_or(""))))
                        .count()
                };
                score(a).cmp(&score(b)).then_with(|| b.name.cmp(&a.name))
            })
            .ok_or_else(|| "no seeded templates are available for an offline build".to_string())?;
        let draft: Draft = serde_json::from_value(to_draft(&template)).map_err(|e| e.to_string())?;
        let flow = assemble(&draft, &self.catalog, goal);
        let issues = crate::validate::validate(&flow, &self.catalog);
        if !issues.is_empty() {
            return Err(format!(
                "offline template is invalid: {}",
                issues.iter().map(ToString::to_string).collect::<Vec<_>>().join("; ")
            ));
        }
        Ok(Compiled {
            outcome: Outcome::Flow {
                flow: Box::new(flow),
                notes: vec![format!("Started from the {} template in offline mode.", template.name)],
                sample: json!({}),
            },
            cost_usd: 0.0,
            repairs: Vec::new(),
        })
    }

    /// Test-runs the saved flow and fixes what fails, at most [`MAX_FIX_ROUNDS`] times (AR-6, AR-9).
    /// Problems left after the last round are logged, not fatal: the person sees them in the editor.
    /// Tests the flow, fixes what fails, then settles every step card.
    async fn test_and_fix(&self, ev: &mut Events<'_>, flow: Flow, sample: &Json, brief: &str) {
        let (flow, fixed, remaining) = self.test_and_fix_rounds(ev, flow, sample, brief).await;
        for (id, event) in progress::finals(&flow, &fixed, &remaining) {
            ev.emit(Some(id), event).await;
        }
    }

    /// Returns the last flow, the steps a fix changed, and the steps with problems left.
    async fn test_and_fix_rounds(
        &self,
        ev: &mut Events<'_>,
        mut flow: Flow,
        sample: &Json,
        brief: &str,
    ) -> (Flow, Vec<String>, Vec<Option<String>>) {
        let models = self.services.models.as_ref();
        let mut fixed: Vec<String> = Vec::new();
        let mut cases = Vec::new();
        if flow.kind == FlowKind::App {
            ev.log("Writing test questions for the app.").await;
            match appkit::suite(models, &flow, brief).await {
                Ok((suite, usd)) => {
                    ev.cost(usd).await;
                    cases = suite;
                }
                Err(e) => tracing::warn!(build_id = %ev.build_id, error = %e, "could not write a test suite"),
            }
        }
        if !cases.is_empty() {
            for patch in progress::tests_card(&flow, cases.len()) {
                ev.emit(Some(progress::TESTS), patch).await;
            }
            ev.emit(Some(progress::TESTS), progress::tests_state(0, cases.len(), 0, false)).await;
        }
        for round in 0..=MAX_FIX_ROUNDS {
            let (passed, failures) = self.test(ev, &flow, sample, &cases).await;
            let failed = u32::try_from(failures.len()).unwrap_or(u32::MAX);
            ev.emit(None, BuildEventKind::TestResult { passed, failed }).await;
            let last = failures.is_empty() || round == MAX_FIX_ROUNDS;
            if !cases.is_empty() {
                let state = progress::tests_state(passed, cases.len(), fixed.len(), last);
                ev.emit(Some(progress::TESTS), state).await;
            }
            let remaining = || failures.iter().map(|f| f.node_id.clone()).collect();
            if failures.is_empty() {
                ev.log("All tests passed.").await;
                return (flow, fixed, vec![]);
            }
            if round == MAX_FIX_ROUNDS {
                ev.log(format!(
                    "{failed} test problem(s) remain after {MAX_FIX_ROUNDS} fixes; check them in the editor."
                ))
                .await;
                return (flow, fixed, remaining());
            }
            ev.log(format!("Fixing {failed} problem(s) the tests found.")).await;
            let revised = match revise(models, &self.catalog, &flow, &fix_request(&failures)).await {
                Ok(revised) => revised,
                Err(e) => {
                    ev.log(format!("Could not fix the failing tests: {e}")).await;
                    return (flow, fixed, remaining());
                }
            };
            ev.cost(revised.cost_usd).await;
            let next = revised.flow;
            if let Err(e) = self.services.store.put_flow(&next).await {
                ev.log(format!("Could not save the fixed flow: {e}")).await;
                return (flow, fixed, remaining());
            }
            let patch = diff(&flow, &next).json_patch;
            ev.emit(None, BuildEventKind::FlowPatch { json_patch: patch }).await;
            let source = if cases.is_empty() { "Test run" } else { "Test suite" };
            for node in next.nodes.iter().filter(|n| flow.node(&n.id).is_none_or(|old| changed(old, n))) {
                let problem = failures.iter().find(|f| f.node_id.as_deref() == Some(&node.id)).or(failures.first());
                let fix = BuildEventKind::Fix {
                    description: problem.map_or_else(String::new, |f| f.problem.clone()),
                    title: Some(format!("Fixed {}", node.name)),
                    before: None,
                    after: None,
                    source: Some(source.into()),
                };
                ev.emit(Some(&node.id), fix).await;
                fixed.push(node.id.clone());
            }
            flow = next;
        }
        (flow, fixed, vec![])
    }

    /// One test round: the app's suite and safety check, or one sample run for an automation.
    async fn test(&self, ev: &mut Events<'_>, flow: &Flow, sample: &Json, cases: &[Case]) -> (u32, Vec<Failure>) {
        if flow.kind == FlowKind::App && !cases.is_empty() {
            let models = self.services.models.as_ref();
            let (passed, mut failures, usd) = appkit::run(models, self.runner.as_ref(), &flow.id, cases).await;
            ev.cost(usd).await;
            failures.extend(appkit::missing_guardrails(flow, &self.catalog));
            return (passed, failures);
        }
        let failures = smoke(self.runner.as_ref(), self.services.store.as_ref(), &flow.id, sample).await;
        (u32::from(failures.is_empty()), failures)
    }

    /// The goal plus attachments and hints, with the text of each Instructions attachment.
    async fn brief(&self, ev: &mut Events<'_>, req: &BuildRequest, instructions: &[(String, String)]) -> String {
        let mut brief = format!("Goal: {}", req.goal.trim());
        if let Some(kind) = req.kind {
            let _ = write!(
                brief,
                "
Build it as: {}",
                json!(kind).as_str().unwrap_or_default()
            );
        }
        if !req.hints.is_empty() {
            let _ = write!(
                brief,
                "
Apps and APIs to use: {}",
                req.hints.join(", ")
            );
        }
        for a in &req.attachments {
            let role = match a.role {
                AttachmentRole::Knowledge => "knowledge to answer from",
                AttachmentRole::Instructions => "instructions to follow",
            };
            let _ = write!(
                brief,
                "
Attachment ({role}): {}",
                a.url.as_deref().unwrap_or(&a.name)
            );
            if let Some((_, text)) = instructions.iter().find(|(name, _)| *name == a.name) {
                let verb = if a.kind == AttachmentKind::Audio { "Transcribed" } else { "Read" };
                ev.log(format!("{verb} {}.", a.name)).await;
                let _ = write!(
                    brief,
                    "
  Text: {text}"
                );
            }
        }
        brief
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
            let title = self.catalog.get(&n.node_type).map(|m| m.title.clone());
            ev.emit(Some(&n.id), progress::node(BuildNodeStatus::Building, title, None, None)).await;
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
