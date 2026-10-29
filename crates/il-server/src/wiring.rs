//! Wires persistent storage and registered implementations into `Services`.

use std::sync::Arc;

use async_trait::async_trait;
use il_core::Flow;
use il_sdk::Services;
use il_sdk::registry::RouteCtx;
use il_sdk::services::metering::MeteredGateway;
use il_sdk::services::{AgentTool, ArchitectService, FlowRunner, FlowToolProvider, SystemClock, ToolError};
use il_sdk::testkit::FakeServices;
use il_store::db::SqliteStore;
use il_store::files::DiskFileStore;

use crate::Config;

/// Shared state for core routes.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub services: Services,
    pub runner: Arc<dyn FlowRunner>,
    pub architect: Arc<dyn ArchitectService>,
    /// The engine itself (event bus for SSE, waits, recovery).
    pub engine: il_engine::Engine,
    /// Starts/stops flow triggers on activate/deactivate; chat/API/MCP entry point.
    pub triggers: Arc<il_engine::triggers::TriggerSupervisor>,
    pub rate_limiter: Arc<crate::rate_limit::RateLimiter>,
    pub(crate) github_secret: Option<String>,
}

struct PublishedFlowTools {
    services: Services,
}

#[async_trait]
impl FlowToolProvider for PublishedFlowTools {
    async fn for_flow(&self, workspace_id: &str, flow: &Flow) -> Result<Vec<Arc<dyn AgentTool>>, ToolError> {
        if flow.workspace_id != workspace_id {
            return Err(ToolError::Failed("flow not found".into()));
        }
        il_connectors::openapi::tool::for_flow(flow, &self.services).await
    }
}

impl AppState {
    /// Context handed to registered route factories.
    pub fn route_ctx(&self) -> RouteCtx {
        RouteCtx {
            services: self.services.clone(),
            runner: self.runner.clone(),
            architect: self.architect.clone(),
            flow_tools: Arc::new(PublishedFlowTools { services: self.services.clone() }),
            config: self.config.raw.clone(),
        }
    }
}

/// Makes sure every implementation crate is linked so its `inventory` registrations exist.
pub fn link_all() {
    il_store::link();
    il_ai::link();
    il_engine::link();
    il_nodes::link();
    il_connectors::link();
    il_media::link();
    il_knowledge::link();
    il_architect::link();
    il_publish::link();
}

/// Builds the app state from config.
pub fn build_state(config: Config) -> AppState {
    link_all();
    let fakes = FakeServices::default();
    finish_state(config, fakes.services(), None)
}

/// Builds the normal server with SQLite and disk files; fake providers remain available offline.
pub async fn build_state_persistent(config: Config) -> anyhow::Result<AppState> {
    link_all();
    let storage =
        config.raw.get("storage").and_then(|s| s.get("path")).and_then(serde_json::Value::as_str).unwrap_or("data");
    let root = std::path::PathBuf::from(storage);
    let db_path = root.join("interlock.db");
    let store = tokio::task::spawn_blocking(move || SqliteStore::open(db_path)).await??;
    let files = DiskFileStore::new(store.clone(), root.join("files")).await?;
    let store = Arc::new(store);
    let mut services = FakeServices::default().services();
    services.store = store.clone();
    services.files = Arc::new(files);
    Ok(finish_state(config, services, Some(store)))
}

/// Wires the real services; `store` (the SQLite store) also backs knowledge bases.
fn finish_state(config: Config, mut services: Services, store: Option<Arc<SqliteStore>>) -> AppState {
    services.clock = Arc::new(SystemClock);
    if let Ok(http) = il_sdk::SafeHttp::new() {
        services.http = http;
    }
    wire_models(&mut services, &config);
    services.models =
        Arc::new(MeteredGateway::new(services.models.clone(), services.store.clone(), services.clock.clone()));
    if let Some(store) = store {
        wire_knowledge(&mut services, store, &config);
    }
    services.flow_tools = Arc::new(PublishedFlowTools { services: services.clone() });
    services.notifier =
        Arc::new(il_connectors::slack::MessagingNotifier::new(services.clone(), services.notifier.clone()));
    match il_media::r#gen::from_config(&config.raw, services.files.clone(), config.providers == "fake") {
        Ok(media) => services.media = media,
        Err(e) => tracing::error!(error = %e, "media provider not started; using offline fake"),
    }
    tracing::info!(providers = %config.providers, "services wired");
    let mut engine_config: il_engine::EngineConfig = config.section("engine").unwrap_or_default();
    engine_config.caps = config.section("caps").unwrap_or_default();
    let engine = il_engine::Engine::new(services.clone(), il_engine::NodeLookup::default(), engine_config);
    let runner: Arc<dyn FlowRunner> = Arc::new(engine.clone());
    let triggers = Arc::new(il_engine::triggers::TriggerSupervisor::new(
        services.clone(),
        runner.clone(),
        il_engine::NodeLookup::default(),
    ));
    let architect = Arc::new(il_architect::Architect::new_with_fake_templates(
        services.clone(),
        runner.clone(),
        il_architect::catalog::Catalog::registered(),
        config.providers == "fake",
    ));
    AppState {
        config: Arc::new(config),
        services,
        runner,
        architect,
        engine,
        triggers,
        rate_limiter: Arc::default(),
        github_secret: std::env::var("GITHUB_CLIENT_SECRET").ok().filter(|s| !s.is_empty()),
    }
}

/// Restarts nonterminal runs and active triggers from the configured store.
pub async fn recover_startup(state: &AppState) -> anyhow::Result<()> {
    reset_expired_demos(state).await?;
    let demos = state.clone();
    tokio::spawn(async move {
        loop {
            demos.services.clock.sleep(std::time::Duration::from_secs(3600)).await;
            if let Err(error) = reset_expired_demos(&demos).await {
                tracing::error!(error = %error, "demo reset scan failed");
            }
        }
    });
    let runs = state.services.store.recoverable_runs().await?;
    for run in runs {
        if matches!(run.status, il_core::RunStatus::Running | il_core::RunStatus::Queued)
            && let Err(error) = state.engine.recover(&run.id).await
        {
            tracing::error!(run_id = %run.id, error = %error, "run recovery failed");
        }
    }
    let flows = state.services.store.active_flows().await?;
    let ids: Vec<_> = flows.into_iter().map(|flow| flow.id).collect();
    let restored = state.triggers.restore(&ids).await;
    tracing::info!(restored, "active flow triggers restored");
    state.engine.expire_due_approvals().await?;
    let engine = state.engine.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if let Err(error) = engine.expire_due_approvals().await {
                tracing::error!(error = %error, "approval expiry scan failed");
            }
        }
    });
    Ok(())
}

/// Knowledge bases on the SQLite store, embedded with the model gateway. Web crawling needs `TAVILY_API_KEY`
/// and stays off in fake mode; the templates' `.example` sites come from `<templates_dir>/../docs`.
fn wire_knowledge(services: &mut Services, store: Arc<SqliteStore>, config: &Config) {
    let tavily = match (config.providers == "fake", il_sdk::SafeHttp::new()) {
        (false, Ok(http)) => il_knowledge::crawl::Tavily::from_env(http)
            .inspect_err(|e| tracing::info!(error = %e, "web crawling off"))
            .ok(),
        _ => None,
    };
    let fixtures = std::path::Path::new(&config.server.templates_dir).parent().unwrap_or(std::path::Path::new("."));
    let knowledge =
        il_knowledge::ingest::Knowledge::new(store, services.files.clone(), services.models.clone(), tavily)
            .with_recorded_sites(fixtures.join("docs"));
    services.knowledge = Arc::new(knowledge);
}

async fn reset_expired_demos(state: &AppState) -> anyhow::Result<()> {
    let cutoff = state.services.clock.now() - chrono::Duration::hours(24);
    for workspace_id in state.services.store.expired_demo_workspaces(cutoff).await? {
        for flow in state.services.store.list_flows(&workspace_id).await? {
            if state.services.store.is_flow_active(&flow.id).await? {
                state.triggers.deactivate(&flow.id).await?;
            }
        }
        state.services.files.delete_workspace_files(&workspace_id).await?;
        state.services.store.delete_demo_workspace(&workspace_id).await?;
        tracing::info!(workspace_id, "expired demo workspace reset");
    }
    Ok(())
}

/// The real model gateway, with its model check run in the background. Keeps the fake (and says so)
/// when the gateway can't start, e.g. without `NEBIUS_API_KEY`, so the UI still serves.
fn wire_models(services: &mut Services, config: &Config) {
    let models = config.raw.get("models").cloned().unwrap_or_default();
    // Fake mode still goes through the gateway (schema checks, repairs) with the offline provider.
    match il_ai::Gateway::from_config(&models, config.providers == "fake") {
        Ok(gateway) => {
            let gateway = Arc::new(gateway);
            // The fake provider offers no real models, so checking the configured ones would only cry wolf.
            if let (false, Ok(rt)) = (config.providers == "fake", tokio::runtime::Handle::try_current()) {
                let g = gateway.clone();
                rt.spawn(async move { g.check().await });
            }
            services.models = gateway;
        }
        Err(e) => tracing::error!(error = %e, "model gateway not started; model calls use the offline fake"),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use il_core::Flow;
    use il_sdk::services::{Clock, FileStore, FlowToolProvider};
    use il_sdk::testkit::{FakeClock, FakeServices};
    use serde_json::json;

    use super::{PublishedFlowTools, build_state, reset_expired_demos};

    #[tokio::test]
    async fn q04_demo_reset_uses_fake_clock_and_keeps_fresh_workspace() {
        let mut state = build_state(crate::Config::from_value(json!({})).unwrap());
        let clock = std::sync::Arc::new(FakeClock::default());
        state.services.clock = clock.clone();
        let start = clock.now();
        state.services.store.register_demo_workspace("ws_demo_old", start).await.unwrap();
        state.services.store.register_demo_workspace("ws_demo_new", start + chrono::Duration::hours(2)).await.unwrap();
        let mut old: Flow = serde_json::from_value(json!({"id":"old","workspaceId":"ws_demo_old","version":0,
            "name":"Old","kind":"automation","nodes":[],"edges":[]}))
        .unwrap();
        state.services.store.put_flow(&old).await.unwrap();
        old.id = "new".into();
        old.workspace_id = "ws_demo_new".into();
        state.services.store.put_flow(&old).await.unwrap();
        let file =
            state.services.files.put_for_workspace("ws_demo_old", "old.txt", "text/plain", "old".into()).await.unwrap();
        clock.advance(Duration::from_secs(25 * 3600));
        reset_expired_demos(&state).await.unwrap();
        assert!(state.services.store.get_flow("old").await.unwrap().is_none());
        assert!(state.services.store.get_flow("new").await.unwrap().is_some());
        assert!(state.services.files.get(&file.id).await.is_err());
    }

    #[tokio::test]
    async fn p01_published_flow_tools_include_selected_openapi_operations_only_for_owner() {
        let fakes = FakeServices::default();
        let spec = json!({"openapi":"3.1.0","servers":[{"url":"https://api.example.com"}],
            "paths":{"/invoices/{id}":{"get":{"operationId":"getInvoice","parameters":[
                {"name":"id","in":"path","required":true,"schema":{"type":"string"}}]}}}});
        let file = fakes.files.put("invoices.json", "application/json", spec.to_string().into()).await.unwrap();
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws_a","version":1,
        "name":"Billing","kind":"app","edges":[],"nodes":[
            {"id":"get","type":"data.openapi","name":"Get invoice",
                "settings":{"spec":file.id,"operationId":"getInvoice"}}
        ]}))
        .unwrap();
        let provider = PublishedFlowTools { services: fakes.services() };
        assert_eq!(provider.for_flow("ws_a", &flow).await.unwrap()[0].name(), "getInvoice");
        assert!(provider.for_flow("ws_other", &flow).await.is_err());
    }
}
