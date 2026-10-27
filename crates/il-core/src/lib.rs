//! Interlock contracts. Every other crate and the web app (via generated TS types) plug into these.
//!
//! Rules: no IO and no async runtime here. Additive changes are fine in any task; breaking
//! changes need a `contract:` PR (AGENTS.md §6).

pub mod contracts;
pub mod dto;
pub mod error;
pub mod events;
pub mod expr;
pub mod flow;
pub mod item;
pub mod manifest;
pub mod run;

pub use error::{ApiError, CoreError};
pub use events::{BuildEvent, BuildEventKind, RunEvent, RunEventKind};
pub use flow::{Edge, Flow, FlowKind, FlowMeta, FlowSettings, Group, Node, PortRef, Position};
pub use item::{FileRef, Item};
pub use manifest::{
    Category, CostHint, CredentialKind, CredentialSpec, ExecMode, NodeExample, NodeManifest, NodeStatus, PortSpec,
    SideEffect,
};
pub use run::{Approval, ApprovalDecision, ApprovalStatus, Decision, NodeRun, Run, RunStatus};

/// Shared JSON value type used across contracts.
pub type Json = serde_json::Value;
