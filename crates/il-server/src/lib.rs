//! The Interlock server: the only crate that wires concrete implementations together.

pub mod api;
pub mod assets;
pub mod config;
pub mod routes;
pub mod wiring;

pub use config::Config;
pub use wiring::{AppState, build_state};

/// Builds the full axum app: core routes, registered route groups, embedded web build.
pub fn app(state: AppState) -> axum::Router {
    routes::router(state)
}
