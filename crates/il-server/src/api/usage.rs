//! Usage: the workspace's spend from the ledger over the last 30 days, and the caps in force.

use std::collections::BTreeMap;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{Duration, NaiveTime, Utc};
use il_core::dto::{UsageCaps, UsageDay, UsageFlow, UsageReport};
use il_engine::caps::CapsConfig;

use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

const DAYS: i64 = 30;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/usage", get(usage))
}

async fn usage(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<UsageReport>> {
    let today = Utc::now().date_naive();
    let first = today - Duration::days(DAYS - 1);
    let since = first.and_time(NaiveTime::MIN).and_utc();
    let store = &state.services.store;
    let entries = store.spend_entries(&ws.0.workspace_id, since).await.map_err(ApiErr::internal)?;

    let mut days: BTreeMap<String, f64> = (0..DAYS).map(|i| ((first + Duration::days(i)).to_string(), 0.0)).collect();
    let mut by_flow: BTreeMap<String, f64> = BTreeMap::new();
    for e in &entries {
        *days.entry(e.at.date_naive().to_string()).or_default() += e.cost_usd;
        if let Some(flow) = &e.flow_id {
            *by_flow.entry(flow.clone()).or_default() += e.cost_usd;
        }
    }
    let mut flows = Vec::new();
    for (flow_id, cost_usd) in by_flow {
        let name = store.get_flow(&flow_id).await.ok().flatten().map_or_else(|| "Deleted flow".into(), |f| f.name);
        flows.push(UsageFlow { flow_id, name, cost_usd });
    }
    flows.sort_by(|a, b| b.cost_usd.total_cmp(&a.cost_usd));
    let caps: CapsConfig = state.config.section("caps").unwrap_or_default();
    Ok(Json(UsageReport {
        total_usd: entries.iter().map(|e| e.cost_usd).sum(),
        days: days.into_iter().map(|(day, cost_usd)| UsageDay { day, cost_usd }).collect(),
        flows,
        caps: UsageCaps {
            run_usd: caps.run_usd,
            flow_daily_usd: caps.flow_daily_usd,
            workspace_daily_usd: caps.workspace_daily_usd,
        },
    }))
}
