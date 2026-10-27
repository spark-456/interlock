//! The engine. Knows no specific node: it runs whatever the registry holds, through il-sdk traits only.

pub mod approvals;
pub mod caps;
pub mod events;
pub mod executor;
pub mod replay;
pub mod triggers;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
