//! Wires concrete implementations into `Services`. Until B01 is wired, fake services from
//! `il_sdk::testkit` stand in.

use std::sync::Arc;

use il_sdk::Services;
use il_sdk::registry::RouteCtx;
use il_sdk::services::{ArchitectService, FlowRunner, SystemClock};
use il_sdk::testkit::FakeServices;

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
}

impl AppState {
    /// Context handed to registered route factories.
    pub fn route_ctx(&self) -> RouteCtx {
        RouteCtx {
            services: self.services.clone(),
            runner: self.runner.clone(),
            architect: self.architect.clone(),
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
    let mut services = fakes.services();
    services.clock = Arc::new(SystemClock);
    if let Ok(http) = il_sdk::SafeHttp::new() {
        services.http = http;
    }
    if config.providers != "fake" {
        wire_models(&mut services, &config);
    }
    tracing::info!(providers = %config.providers, "services wired (in-memory store until B01 lands)");
    let mut engine_config: il_engine::EngineConfig = config.section("engine").unwrap_or_default();
    engine_config.caps = config.section("caps").unwrap_or_default();
    let engine = il_engine::Engine::new(services.clone(), il_engine::NodeLookup::default(), engine_config);
    let runner: Arc<dyn FlowRunner> = Arc::new(engine.clone());
    let triggers = Arc::new(il_engine::triggers::TriggerSupervisor::new(
        services.clone(),
        runner.clone(),
        il_engine::NodeLookup::default(),
    ));
    let architect = Arc::new(il_architect::Architect::new(services.clone(), il_architect::catalog::Catalog::registered()));
    AppState { config: Arc::new(config), services, runner, architect, engine, triggers }
}

/// The real model gateway, with its model check run in the background. Keeps the fake (and says so)
/// when the gateway can't start, e.g. without `NEBIUS_API_KEY`, so the UI still serves.
fn wire_models(services: &mut Services, config: &Config) {
    let models = config.raw.get("models").cloned().unwrap_or_default();
    match il_ai::Gateway::from_config(&models, false) {
        Ok(gateway) => {
            let gateway = Arc::new(gateway);
            if let Ok(rt) = tokio::runtime::Handle::try_current() {
                let g = gateway.clone();
                rt.spawn(async move { g.check().await });
            }
            services.models = gateway;
        }
        Err(e) => tracing::error!(error = %e, "model gateway not started; model calls use the offline fake"),
    }
}
