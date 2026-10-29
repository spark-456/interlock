//! Usage: the workspace's spend from the ledger over the last 30 days, and the caps in force.

use std::collections::BTreeMap;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{Duration, NaiveTime, Utc};
use il_core::ApiError;
use il_core::dto::{UsageCaps, UsageDay, UsageFlow, UsageReport, UsageRole};
use il_engine::caps::{self, CapsConfig};
use serde_json::json;

use super::{ApiErr, ApiResult, Ws};
use crate::wiring::AppState;

const DAYS: i64 = 30;

pub(super) fn routes() -> Router<AppState> {
    Router::new().route("/api/usage", get(usage)).route("/api/usage/caps", put(update_caps))
}

async fn update_caps(State(state): State<AppState>, ws: Ws, Json(caps): Json<UsageCaps>) -> ApiResult<Json<UsageCaps>> {
    if ws.0.demo {
        return Err(ApiErr(StatusCode::FORBIDDEN, ApiError::new("demo_restricted", "Demo caps cannot be changed.")));
    }
    let config: CapsConfig = state.config.section("caps").unwrap_or_default();
    if !caps::valid_workspace_caps(&caps, config.global_daily_usd) {
        return Err(ApiErr::bad_request(
            "Caps must be positive, ordered per run ≤ flow ≤ workspace, and at most the global daily cap.",
        ));
    }
    state.services.store.kv_put(&caps::workspace_caps_key(ws.id()), &json!(caps)).await.map_err(ApiErr::internal)?;
    Ok(Json(caps))
}

async fn usage(State(state): State<AppState>, ws: Ws) -> ApiResult<Json<UsageReport>> {
    let today = Utc::now().date_naive();
    let first = today - Duration::days(DAYS - 1);
    let since = first.and_time(NaiveTime::MIN).and_utc();
    let store = &state.services.store;
    let entries = store.spend_entries(&ws.0.workspace_id, since).await.map_err(ApiErr::internal)?;

    let mut days: BTreeMap<String, f64> = (0..DAYS).map(|i| ((first + Duration::days(i)).to_string(), 0.0)).collect();
    let mut by_flow: BTreeMap<String, f64> = BTreeMap::new();
    let mut by_role: BTreeMap<String, f64> = BTreeMap::new();
    for e in &entries {
        *days.entry(e.at.date_naive().to_string()).or_default() += e.cost_usd;
        if let Some(flow) = &e.flow_id {
            *by_flow.entry(flow.clone()).or_default() += e.cost_usd;
        }
        *by_role.entry(e.role.clone()).or_default() += e.cost_usd;
    }
    let mut flows = Vec::new();
    for (flow_id, cost_usd) in by_flow {
        let name = store.get_flow(&flow_id).await.ok().flatten().map_or_else(|| "Deleted flow".into(), |f| f.name);
        flows.push(UsageFlow { flow_id, name, cost_usd });
    }
    flows.sort_by(|a, b| b.cost_usd.total_cmp(&a.cost_usd));
    let mut roles: Vec<_> = by_role.into_iter().map(|(role, cost_usd)| UsageRole { role, cost_usd }).collect();
    roles.sort_by(|a, b| b.cost_usd.total_cmp(&a.cost_usd).then_with(|| a.role.cmp(&b.role)));
    let config: CapsConfig = state.config.section("caps").unwrap_or_default();
    let caps = caps::effective_caps(store.as_ref(), ws.id(), &config).await;
    Ok(Json(UsageReport {
        total_usd: entries.iter().map(|e| e.cost_usd).sum(),
        days: days.into_iter().map(|(day, cost_usd)| UsageDay { day, cost_usd }).collect(),
        flows,
        roles,
        caps: UsageCaps {
            run_usd: caps.run_usd,
            flow_daily_usd: caps.flow_daily_usd,
            workspace_daily_usd: caps.workspace_daily_usd,
        },
    }))
}
