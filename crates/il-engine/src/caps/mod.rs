//! Spend caps per run, per flow per day, per workspace per day, and global per day (EN-6), and the
//! tighter limits for demo workspaces (WS-2): a smaller run cap and only the cheaper model roles.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use il_core::{FileRef, Flow, Run, dto::UsageCaps};
use il_sdk::services::Store;
use il_sdk::services::models::{
    ChatRequest, ChatResponse, ChatStream, ModelError, ModelGateway, ModelRole, Transcript,
};
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
    /// Run cap in demo workspaces.
    pub demo_run_usd: f64,
    /// Model roles demo runs may use (`fast`, `agent`, `architect`).
    pub demo_models: Vec<String>,
}

impl Default for CapsConfig {
    fn default() -> Self {
        Self {
            run_usd: 1.0,
            flow_daily_usd: 3.0,
            workspace_daily_usd: 5.0,
            global_daily_usd: 25.0,
            demo_run_usd: 0.10,
            demo_models: vec!["fast".into(), "agent".into()],
        }
    }
}

/// Whether a workspace is a demo session's (`ws_demo_…`).
pub fn is_demo(workspace_id: &str) -> bool {
    workspace_id.starts_with("ws_demo")
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

/// Key for a workspace's cap override.
pub fn workspace_caps_key(workspace_id: &str) -> String {
    format!("us:caps:{workspace_id}")
}

/// Validates editable caps against the configured global ceiling.
pub fn valid_workspace_caps(caps: &UsageCaps, global_daily_usd: f64) -> bool {
    [caps.run_usd, caps.flow_daily_usd, caps.workspace_daily_usd, global_daily_usd]
        .iter()
        .all(|value| value.is_finite() && *value > 0.0)
        && caps.run_usd <= caps.flow_daily_usd
        && caps.flow_daily_usd <= caps.workspace_daily_usd
        && caps.workspace_daily_usd <= global_daily_usd
}

/// Workspace overrides take precedence over the config's run, flow, and workspace limits.
pub async fn effective_caps(store: &dyn Store, workspace_id: &str, cfg: &CapsConfig) -> CapsConfig {
    let mut result = cfg.clone();
    let override_caps = store
        .kv_get(&workspace_caps_key(workspace_id))
        .await
        .ok()
        .flatten()
        .and_then(|value| serde_json::from_value::<UsageCaps>(value).ok());
    if let Some(caps) = override_caps.filter(|caps| valid_workspace_caps(caps, cfg.global_daily_usd)) {
        result.run_usd = caps.run_usd;
        result.flow_daily_usd = caps.flow_daily_usd;
        result.workspace_daily_usd = caps.workspace_daily_usd;
    }
    result
}

/// Returns the first cap that is reached, if any. Store errors never block a run.
pub async fn check(store: &dyn Store, now: DateTime<Utc>, run: &Run, flow: &Flow, cfg: &CapsConfig) -> Option<CapHit> {
    let cfg = effective_caps(store, &run.workspace_id, cfg).await;
    let day = start_of_day(now);
    let mut run_cap = flow.settings.spend_cap_usd_per_run.unwrap_or(cfg.run_usd);
    if is_demo(&run.workspace_id) {
        run_cap = run_cap.min(cfg.demo_run_usd);
    }
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

/// The model gateway for demo runs: roles outside `allowed` use the next cheaper allowed role
/// (`architect` → `agent` → `fast`).
pub struct DemoModels {
    pub inner: Arc<dyn ModelGateway>,
    pub allowed: Vec<String>,
}

impl DemoModels {
    fn role(&self, role: ModelRole) -> ModelRole {
        let ok = |r: &str| self.allowed.iter().any(|a| a == r);
        match role {
            ModelRole::Architect if ok("architect") => ModelRole::Architect,
            ModelRole::Architect | ModelRole::Agent if ok("agent") => ModelRole::Agent,
            _ => ModelRole::Fast,
        }
    }
}

#[async_trait]
impl ModelGateway for DemoModels {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        self.inner.chat(self.role(role), req).await
    }
    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        self.inner.chat_stream(self.role(role), req).await
    }
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        self.inner.embed(texts).await
    }
    async fn transcribe(&self, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
        self.inner.transcribe(file, bytes).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use chrono::Utc;
    use il_core::{Flow, Run};
    use il_sdk::services::models::{ChatRequest, ModelGateway, ModelRole};
    use il_sdk::{
        services::{Store, store::SpendEntry},
        testkit::{FakeGateway, MemoryStore},
    };
    use serde_json::json;

    use super::{DemoModels, is_demo};

    #[tokio::test]
    async fn u02_workspace_cap_override_precedes_config() {
        let store = MemoryStore::default();
        let now = Utc::now();
        store.kv_put("us:caps:ws", &json!({"runUsd":0.1,"flowDailyUsd":0.2,"workspaceDailyUsd":0.4})).await.unwrap();
        store
            .record_spend(&SpendEntry {
                workspace_id: "ws".into(),
                flow_id: None,
                run_id: None,
                build_id: None,
                role: "agent".into(),
                cost_usd: 0.5,
                at: now,
            })
            .await
            .unwrap();
        let flow: Flow = serde_json::from_value(json!({"id":"flow","workspaceId":"ws","version":1,
            "name":"Flow","kind":"automation","nodes":[],"edges":[]}))
        .unwrap();
        let run: Run = serde_json::from_value(json!({"id":"run","flowId":"flow","workspaceId":"ws",
            "flowVersion":1,"status":"running","trigger":"test","startedAt":now,"finishedAt":null,
            "costUsd":0,"error":null}))
        .unwrap();
        let hit = super::check(&store, now, &run, &flow, &super::CapsConfig::default()).await.unwrap();
        assert_eq!(hit.cap, "workspace_daily");
        assert_eq!(hit.limit_usd, 0.4);
    }

    #[tokio::test]
    async fn b10_demo_runs_use_only_the_allowed_roles() {
        let fake = Arc::new(FakeGateway::default());
        let demo = DemoModels { inner: fake.clone(), allowed: vec!["fast".into(), "agent".into()] };
        for role in [ModelRole::Architect, ModelRole::Agent, ModelRole::Fast] {
            fake.push_text("ok");
            demo.chat(role, ChatRequest::default()).await.unwrap();
        }
        let roles: Vec<ModelRole> = fake.calls().into_iter().map(|(r, _)| r).collect();
        assert_eq!(roles, [ModelRole::Agent, ModelRole::Agent, ModelRole::Fast]);

        let fast_only = DemoModels { inner: fake.clone(), allowed: vec!["fast".into()] };
        fake.push_text("ok");
        fast_only.chat(ModelRole::Architect, ChatRequest::default()).await.unwrap();
        assert_eq!(fake.calls().last().unwrap().0, ModelRole::Fast);
        assert!(is_demo("ws_demo_01J") && !is_demo("ws_local"));
    }
}
