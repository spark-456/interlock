//! Test-and-fix (AR-6): run the saved flow once with sample data through the engine, collect what
//! failed, and turn the failures into a fix request for [`crate::compile::revise`].

use il_core::{Item, Json, RunStatus};
use il_sdk::services::runner::{FlowRunner, StartRun};
use il_sdk::services::store::Store;

/// Fix rounds after the first test run.
pub const MAX_FIX_ROUNDS: usize = 2;

/// Something a test found wrong, optionally pinned to a step.
#[derive(Debug, Clone, PartialEq)]
pub struct Failure {
    pub node_id: Option<String>,
    pub problem: String,
}

/// Test-runs `flow_id` with `sample` as the trigger's item. Test runs never post or send for real.
pub async fn smoke(runner: &dyn FlowRunner, store: &dyn Store, flow_id: &str, sample: &Json) -> Vec<Failure> {
    let req = StartRun {
        flow_id: flow_id.into(),
        version: None,
        trigger: "test".into(),
        items: vec![Item::json(sample.clone())],
        trigger_node: None,
    };
    let outcome = match runner.invoke(req).await {
        Ok(outcome) => outcome,
        Err(e) => return vec![Failure { node_id: None, problem: e.to_string() }],
    };
    if !matches!(outcome.run.status, RunStatus::Failed | RunStatus::CapReached) {
        return Vec::new();
    }
    let failed: Vec<_> = store
        .node_runs(&outcome.run.id)
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|n| n.status == RunStatus::Failed)
        .map(|n| Failure { node_id: Some(n.node_id), problem: n.error.unwrap_or_else(|| "it failed".into()) })
        .collect();
    if failed.is_empty() {
        let problem = outcome.run.error.unwrap_or_else(|| "the test run failed".into());
        return vec![Failure { node_id: None, problem }];
    }
    failed
}

/// The request that asks the model to fix `failures`.
pub fn fix_request(failures: &[Failure]) -> String {
    let lines: Vec<String> = failures
        .iter()
        .map(|f| match &f.node_id {
            Some(id) => format!("- step `{id}`: {}", f.problem),
            None => format!("- {}", f.problem),
        })
        .collect();
    format!(
        "A test run with sample data found these problems:\n{}\nFix the flow so they no longer happen; for AI steps, \
         improve the prompts and instructions.",
        lines.join("\n")
    )
}
