//! The engine. Knows no specific node: it runs whatever the registry holds, through il-sdk traits only.
//!
//! Entry point: [`Engine`], which implements `il_sdk::services::FlowRunner`.

pub mod approvals;
pub mod caps;
pub mod events;
pub mod executor;
pub mod replay;
pub mod state;
pub mod triggers;

pub use events::EventBus;
pub use executor::{Engine, EngineConfig, EngineError, NodeLookup};

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
