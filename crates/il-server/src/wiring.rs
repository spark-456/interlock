//! Wires concrete implementations into `Services`. Until B01/B02/B03/A01 land, fake services
//! from `il_sdk::testkit` stand in and the runner/architect report "not wired yet".

use std::sync::Arc;

use async_trait::async_trait;
use il_core::Flow;
use il_core::dto::{AskDiff, BuildAnswers, BuildRequest};
use il_sdk::Services;
use il_sdk::registry::RouteCtx;
use il_sdk::services::architect::ArchitectError;
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
    AppState { config: Arc::new(config), services, runner, architect: Arc::new(NotWired), engine, triggers }
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

/// Placeholder until the architect (A01) is wired. (The runner is the engine.)
struct NotWired;

const ARCHITECT: &str = "the architect is not wired yet (task A01)";

#[async_trait]
impl ArchitectService for NotWired {
    async fn start_build(&self, _ws: &str, _req: BuildRequest) -> Result<String, ArchitectError> {
        Err(ArchitectError::Failed(ARCHITECT.into()))
    }
    async fn answer(&self, _build_id: &str, _answers: BuildAnswers) -> Result<(), ArchitectError> {
        Err(ArchitectError::Failed(ARCHITECT.into()))
    }
    async fn ask(&self, _flow: &Flow, _message: &str) -> Result<AskDiff, ArchitectError> {
        Err(ArchitectError::Failed(ARCHITECT.into()))
    }
}
