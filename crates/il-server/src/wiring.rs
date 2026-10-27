//! Wires concrete implementations into `Services`. Until B01/B02/B03/A01 land, fake services
//! from `il_sdk::testkit` stand in and the runner/architect report "not wired yet".

use std::sync::Arc;

use async_trait::async_trait;
use il_core::dto::{AskDiff, BuildAnswers, BuildRequest};
use il_core::{ApprovalDecision, Flow};
use il_sdk::Services;
use il_sdk::registry::RouteCtx;
use il_sdk::services::architect::ArchitectError;
use il_sdk::services::runner::InvokeOutcome;
use il_sdk::services::{ArchitectService, FlowRunner, RunnerError, StartRun, SystemClock};
use il_sdk::testkit::FakeServices;

use crate::Config;

/// Shared state for core routes.
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub services: Services,
    pub runner: Arc<dyn FlowRunner>,
    pub architect: Arc<dyn ArchitectService>,
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
    tracing::info!(providers = %config.providers, "services wired (T00: in-memory fakes until B01/B02 land)");
    AppState { config: Arc::new(config), services, runner: Arc::new(NotWired), architect: Arc::new(NotWired) }
}

/// Placeholder until the engine (B03) and architect (A01) are wired.
struct NotWired;

const ENGINE: &str = "the engine is not wired yet (task B03)";
const ARCHITECT: &str = "the architect is not wired yet (task A01)";

#[async_trait]
impl FlowRunner for NotWired {
    async fn start(&self, _req: StartRun) -> Result<String, RunnerError> {
        Err(RunnerError::Other(ENGINE.into()))
    }
    async fn invoke(&self, _req: StartRun) -> Result<InvokeOutcome, RunnerError> {
        Err(RunnerError::Other(ENGINE.into()))
    }
    async fn decide(&self, _id: &str, _d: ApprovalDecision, _by: Option<String>) -> Result<(), RunnerError> {
        Err(RunnerError::Other(ENGINE.into()))
    }
    async fn cancel(&self, _run_id: &str) -> Result<(), RunnerError> {
        Err(RunnerError::Other(ENGINE.into()))
    }
    async fn replay(&self, _id: &str) -> Result<String, RunnerError> {
        Err(RunnerError::Other(ENGINE.into()))
    }
}

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
