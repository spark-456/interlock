//! Interlock SDK: everything a node, connector, provider or route group plugs into.
//!
//! Dependency rule: `il-core ← il-sdk ← implementations`. Engine, architect and publish use
//! only the traits here; only `il-server` wires concrete crates together.

pub mod ctx;
pub mod http;
pub mod node;
pub mod poll;
pub mod prelude;
pub mod registry;
pub mod services;
pub mod validate;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub use ctx::{ApprovalSpec, NodeCtx, RunInfo, TriggerCtx, TriggerHandle};
pub use http::{HttpError, SafeHttp};
pub use node::{NodeError, NodeExecutor, NodeInput, NodeOutput, TriggerNode, ValidationIssue};
pub use services::Services;

/// Re-exported so node crates can use `il_sdk::inventory::submit!`.
pub use inventory;

/// New prefixed ULID, e.g. `id("run")` → `run_01J…`.
pub fn id(prefix: &str) -> String {
    format!("{prefix}_{}", ulid::Ulid::new())
}
