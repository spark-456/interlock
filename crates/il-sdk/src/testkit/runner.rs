//! Fake `FlowRunner` (records starts) and `ArchitectService` (returns fixed ids) for trigger,
//! route and publish tests.

use std::sync::Mutex;

use async_trait::async_trait;
use il_core::dto::{AskDiff, BuildAnswers, BuildRequest};
use il_core::{ApprovalDecision, Flow, Item, Run, RunStatus};
use serde_json::{Value, json};

use crate::services::architect::{ArchitectError, ArchitectService};
use crate::services::runner::{FlowRunner, InvokeOutcome, RunnerError, StartRun};

/// Records every start/invoke; `invoke` echoes the input items as the response.
#[derive(Default)]
pub struct FakeRunner {
    started: Mutex<Vec<StartRun>>,
}

impl FakeRunner {
    /// Every `StartRun` received so far.
    pub fn started(&self) -> Vec<StartRun> {
        self.started.lock().map(|s| s.clone()).unwrap_or_default()
    }

    fn record(&self, req: StartRun) -> String {
        let mut s = self.started.lock().unwrap_or_else(|e| e.into_inner());
        s.push(req);
        format!("run_fake_{}", s.len())
    }
}

#[async_trait]
impl FlowRunner for FakeRunner {
    async fn start(&self, req: StartRun) -> Result<String, RunnerError> {
        Ok(self.record(req))
    }
    async fn invoke(&self, req: StartRun) -> Result<InvokeOutcome, RunnerError> {
        let response: Vec<Item> = req.items.clone();
        let flow_id = req.flow_id.clone();
        let id = self.record(req);
        let run = Run {
            id,
            flow_id,
            workspace_id: "ws_test".into(),
            flow_version: 1,
            status: RunStatus::Succeeded,
            trigger: "test".into(),
            started_at: chrono::Utc::now(),
            finished_at: Some(chrono::Utc::now()),
            cost_usd: 0.0,
            error: None,
            replay: false,
            state: Value::Null,
        };
        Ok(InvokeOutcome { run, response })
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

/// Returns `bld_fake` for builds and an empty diff for Ask.
#[derive(Default)]
pub struct FakeArchitect;

#[async_trait]
impl ArchitectService for FakeArchitect {
    async fn start_build(&self, _ws: &str, _req: BuildRequest) -> Result<String, ArchitectError> {
        Ok("bld_fake".into())
    }
    async fn answer(&self, _id: &str, _answers: BuildAnswers) -> Result<(), ArchitectError> {
        Ok(())
    }
    async fn ask(&self, _flow: &Flow, _message: &str) -> Result<AskDiff, ArchitectError> {
        Ok(AskDiff { summary: "no changes".into(), json_patch: json!([]) })
    }
}
