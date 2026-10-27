//! Spend caps per run, per flow per day, per workspace per day, and global per day (EN-6).

use chrono::{DateTime, Utc};
use il_core::{Flow, Run};
use il_sdk::services::Store;
use il_sdk::services::store::SpendScope;
use serde::{Deserialize, Serialize};

/// Cap limits in USD. Read from the `[caps]` config section.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CapsConfig {
    pub run_usd: f64,
    pub flow_daily_usd: f64,
    pub workspace_daily_usd: f64,
    pub global_daily_usd: f64,
}

impl Default for CapsConfig {
    fn default() -> Self {
        Self { run_usd: 1.0, flow_daily_usd: 3.0, workspace_daily_usd: 5.0, global_daily_usd: 25.0 }
    }
}

/// A cap that has been reached.
#[derive(Debug, Clone, PartialEq)]
pub struct CapHit {
    /// `run`, `flow_daily`, `workspace_daily` or `global_daily`.
    pub cap: &'static str,
    pub limit_usd: f64,
    pub spent_usd: f64,
}

fn start_of_day(now: DateTime<Utc>) -> DateTime<Utc> {
    now.date_naive().and_hms_opt(0, 0, 0).map(|d| d.and_utc()).unwrap_or(now)
}

/// Returns the first cap that is reached, if any. Store errors never block a run.
pub async fn check(store: &dyn Store, now: DateTime<Utc>, run: &Run, flow: &Flow, cfg: &CapsConfig) -> Option<CapHit> {
    let day = start_of_day(now);
    let run_cap = flow.settings.spend_cap_usd_per_run.unwrap_or(cfg.run_usd);
    let checks: [(&'static str, f64, SpendScope); 4] = [
        ("run", run_cap, SpendScope::Run(run.id.clone())),
        ("flow_daily", cfg.flow_daily_usd, SpendScope::FlowSince(flow.id.clone(), day)),
        ("workspace_daily", cfg.workspace_daily_usd, SpendScope::WorkspaceSince(run.workspace_id.clone(), day)),
        ("global_daily", cfg.global_daily_usd, SpendScope::GlobalSince(day)),
    ];
    for (cap, limit, scope) in checks {
        let spent = store.spend(&scope).await.unwrap_or(0.0);
        if spent >= limit {
            return Some(CapHit { cap, limit_usd: limit, spent_usd: spent });
        }
    }
    None
}
