//! The model gateway. Model ids live only in config; code asks for a role.

mod config;
mod fallback;
mod gateway;
pub mod providers;

pub use config::{GatewayConfig, Roles};
pub use gateway::Gateway;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
