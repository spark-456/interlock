//! The Interlock server: the only crate that wires concrete implementations together.

pub mod api;
pub mod assets;
pub mod bundle;
pub mod config;
pub mod rate_limit;
pub mod routes;
pub mod wiring;

pub use config::Config;
pub use routes::router as app;
pub use wiring::{AppState, build_state};
