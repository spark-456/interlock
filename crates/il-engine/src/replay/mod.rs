//! Replay a recorded run (EN-8): copies its events into a new run marked `replay: true`, with
//! the original timing (gaps capped at 1.5 s), so demo mode can show a full run with no model
//! or media calls.

use std::time::Duration;

use il_core::{Run, RunEventKind, RunStatus};

use crate::executor::{Engine, EngineError};

/// Longest pause between replayed events.
const MAX_GAP: Duration = Duration::from_millis(1500);

/// Starts replaying `recorded_run_id`; returns the new run id.
pub async fn replay(engine: &Engine, recorded_run_id: &str) -> Result<String, EngineError> {
    let inner = engine.inner.clone();
    let store = inner.services.store.clone();
    let source = store
        .get_run(recorded_run_id)
        .await
        .map_err(|e| EngineError::Store(e.to_string()))?
        .ok_or_else(|| EngineError::RunNotFound(recorded_run_id.into()))?;
    let events = store.run_events(recorded_run_id, 0).await.map_err(|e| EngineError::Store(e.to_string()))?;
    let now = inner.services.clock.now();
    let mut run = Run {
        id: il_sdk::id("run"),
        status: RunStatus::Running,
        trigger: "replay".into(),
        started_at: now,
        finished_at: None,
        cost_usd: 0.0,
        error: None,
        replay: true,
        ..source
    };
    store.put_run(&run).await.map_err(|e| EngineError::Store(e.to_string()))?;
    inner.set_status(&run.id, RunStatus::Running);
    let id = run.id.clone();

    tokio::spawn(async move {
        let mut prev: Option<chrono::DateTime<chrono::Utc>> = None;
        let mut status = RunStatus::Succeeded;
        for ev in events {
            if let Some(p) = prev {
                let gap = (ev.ts - p).to_std().unwrap_or_default().min(MAX_GAP);
                inner.services.clock.sleep(gap).await;
            }
            prev = Some(ev.ts);
            status = match &ev.kind {
                RunEventKind::RunFailed { .. } => RunStatus::Failed,
                RunEventKind::RunCancelled {} => RunStatus::Cancelled,
                RunEventKind::RunSucceeded { cost_usd, .. } => {
                    run.cost_usd = *cost_usd;
                    RunStatus::Succeeded
                }
                _ => status,
            };
            inner.bus.emit(&run.id, ev.node_id.as_deref(), ev.kind).await;
        }
        run.status = status;
        run.finished_at = Some(inner.services.clock.now());
        let _ = inner.services.store.put_run(&run).await;
        inner.set_status(&run.id, status);
    });
    Ok(id)
}
