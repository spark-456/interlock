//! `trigger.schedule` — Schedule: starts the flow on a five-field cron in a timezone
//! (e.g. `0 9 * * MON` in `Asia/Kolkata`). Sleeps on the service clock, so tests drive it
//! with `FakeClock`.

use std::str::FromStr;

use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use il_sdk::prelude::*;
use il_sdk::services::StartRun;
use il_sdk::{TriggerCtx, TriggerHandle, ValidationIssue};

/// Schedule node.
#[derive(Debug, Default)]
pub struct TriggerSchedule;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.schedule",
        Category::Trigger,
        "Schedule",
        "Starts the flow on a cron schedule in a timezone, e.g. every Monday at 9:00.",
    )
    .icon("clock")
    .trigger()
    .status(NodeStatus::Ready)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "cron"
        ],
        "properties": {
            "cron": {
                "type": "string",
                "title": "Cron",
                "description": "Five-field cron, e.g. `0 9 * * MON`."
            },
            "timezone": {
                "type": "string",
                "title": "Timezone",
                "default": "UTC"
            }
        }
    }))
    .example(
        "Every Monday 9:00",
        json!({
            "cron": "0 9 * * MON",
            "timezone": "Asia/Kolkata"
        }),
    )
});

/// Next fire time strictly after `after`. Accepts five fields (minute precision) or six
/// (with seconds). Day-of-week names (`MON`) are recommended; numbers follow the `cron` crate.
pub fn next_fire(expr: &str, timezone: &str, after: DateTime<Utc>) -> Result<DateTime<Utc>, NodeError> {
    let fields = expr.split_whitespace().count();
    let full = if fields == 5 { format!("0 {expr}") } else { expr.to_string() };
    let schedule = cron::Schedule::from_str(&full)
        .map_err(|e| NodeError::InvalidSettings(vec![ValidationIssue::new("/cron", &format!("invalid cron: {e}"))]))?;
    let tz: Tz = timezone.parse().map_err(|_| {
        NodeError::InvalidSettings(vec![ValidationIssue::new("/timezone", &format!("unknown timezone `{timezone}`"))])
    })?;
    schedule
        .after(&after.with_timezone(&tz))
        .next()
        .map(|t| t.with_timezone(&Utc))
        .ok_or_else(|| NodeError::InvalidSettings(vec![ValidationIssue::new("/cron", "the schedule never fires")]))
}

fn settings(v: &Value) -> (String, String) {
    let cron = v.get("cron").and_then(Value::as_str).unwrap_or_default().to_string();
    let tz = v.get("timezone").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or("UTC").to_string();
    (cron, tz)
}

#[async_trait]
impl NodeExecutor for TriggerSchedule {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    fn validate(&self, settings_value: &Value) -> Result<(), Vec<ValidationIssue>> {
        il_sdk::validate::default_schema_validate(&MANIFEST, settings_value)?;
        let (cron, tz) = settings(settings_value);
        match next_fire(&cron, &tz, Utc::now()) {
            Err(NodeError::InvalidSettings(issues)) => Err(issues),
            _ => Ok(()),
        }
    }

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerSchedule {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        let (cron, tz) = settings(&ctx.settings);
        next_fire(&cron, &tz, ctx.services.clock.now())?;
        let cancel = ctx.cancel.clone();
        let task = tokio::spawn(async move {
            loop {
                let now = ctx.services.clock.now();
                let Ok(next) = next_fire(&cron, &tz, now) else { break };
                let wait = (next - now).to_std().unwrap_or_default();
                tokio::select! {
                    _ = ctx.cancel.cancelled() => break,
                    _ = ctx.services.clock.sleep(wait) => {}
                }
                let req = StartRun {
                    flow_id: ctx.flow_id.clone(),
                    version: None,
                    trigger: MANIFEST.type_id.clone(),
                    items: vec![Item::json(json!({ "firedAt": next.to_rfc3339(), "timezone": tz }))],
                    trigger_node: Some(ctx.node_id.clone()),
                };
                if let Err(e) = ctx.runner.start(req).await {
                    tracing::warn!(flow_id = %ctx.flow_id, error = %e, "scheduled run failed to start");
                }
            }
        });
        Ok(TriggerHandle::new(cancel, Some(task)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerSchedule)) }

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use il_sdk::testkit::{FakeRunner, FakeServices};
    use tokio_util::sync::CancellationToken;

    use super::*;

    fn at(s: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
    }

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&TriggerSchedule);
    }

    #[test]
    fn next_fire_respects_timezone() {
        // 2026-10-26 is a Monday. 09:00 in Kolkata is 03:30 UTC.
        assert_eq!(
            next_fire("0 9 * * MON", "Asia/Kolkata", at("2026-10-26T09:00:00Z")).unwrap(),
            at("2026-11-02T03:30:00Z")
        );
        assert_eq!(next_fire("0 9 * * MON", "UTC", at("2026-10-26T08:59:00Z")).unwrap(), at("2026-10-26T09:00:00Z"));
        assert_eq!(next_fire("*/15 * * * *", "UTC", at("2026-10-26T09:00:00Z")).unwrap(), at("2026-10-26T09:15:00Z"));
    }

    #[test]
    fn rejects_bad_settings() {
        assert!(TriggerSchedule.validate(&json!({ "cron": "nope" })).is_err());
        assert!(TriggerSchedule.validate(&json!({ "cron": "0 9 * * MON", "timezone": "Mars/Base" })).is_err());
        assert!(TriggerSchedule.validate(&json!({ "cron": "0 9 * * MON", "timezone": "Asia/Kolkata" })).is_ok());
    }

    #[tokio::test]
    async fn fires_on_the_fake_clock_and_stops() {
        let fakes = FakeServices::default(); // clock starts Mon 2026-10-26 09:00 UTC
        let runner = Arc::new(FakeRunner::default());
        let ctx = TriggerCtx {
            flow_id: "flw".into(),
            workspace_id: "ws".into(),
            node_id: "n_sched".into(),
            settings: json!({ "cron": "0 * * * *", "timezone": "UTC" }),
            credential_ids: Default::default(),
            services: fakes.services(),
            runner: runner.clone(),
            cancel: CancellationToken::new(),
        };
        let handle = TriggerSchedule.start(ctx).await.unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert!(runner.started().is_empty());
        for expected in 1..=2 {
            fakes.clock.advance(Duration::from_secs(3600));
            for _ in 0..100 {
                if runner.started().len() >= expected {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            assert_eq!(runner.started().len(), expected);
        }
        let first = &runner.started()[0];
        assert_eq!(first.items[0].json["firedAt"], "2026-10-26T10:00:00+00:00");
        assert_eq!(first.trigger_node.as_deref(), Some("n_sched"));
        handle.stop().await;
        fakes.clock.advance(Duration::from_secs(7200));
        tokio::time::sleep(Duration::from_millis(30)).await;
        assert_eq!(runner.started().len(), 2, "stopped triggers do not fire");
    }
}
