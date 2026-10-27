//! Built-in nodes. Each node self-registers with `inventory::submit!`; never import il-store or il-server.

pub mod ai;
pub mod data;
pub mod human;
pub mod knowledge;
pub mod logic;
pub mod output;
pub mod trigger;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
