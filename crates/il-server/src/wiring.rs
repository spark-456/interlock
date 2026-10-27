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
    tracing::info!(providers = %config.providers, "services wired (in-memory fakes until B01/B02 land)");
    let mut engine_config: il_engine::EngineConfig = config.section("engine").unwrap_or_default();
    engine_config.caps = config.section("caps").unwrap_or_default();
    let engine = il_engine::Engine::new(services.clone(), il_engine::NodeLookup::default(), engine_config);
    AppState {
        config: Arc::new(config),
        services,
        runner: Arc::new(engine.clone()),
        architect: Arc::new(NotWired),
        engine,
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
