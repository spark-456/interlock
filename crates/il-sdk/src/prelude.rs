//! `use il_sdk::prelude::*;` — what a node module needs.

pub use std::sync::LazyLock;

pub use async_trait::async_trait;
pub use serde_json::{Value, json};

pub use il_core::manifest::{Category, CostHint, CredentialKind, CredentialSpec, NodeManifest, NodeStatus};
pub use il_core::{FileRef, Item};

pub use crate::ctx::NodeCtx;
pub use crate::node::{NodeError, NodeExecutor, NodeInput, NodeOutput, TriggerNode};
pub use crate::registry::NodeFactory;

/// Shorthand for a credential requirement.
pub fn cred(key: &str, app: &str, kind: CredentialKind, label: &str, scopes: &[&str]) -> CredentialSpec {
    CredentialSpec {
        key: key.into(),
        app: app.into(),
        kind,
        label: label.into(),
        scopes: scopes.iter().map(|s| s.to_string()).collect(),
        optional: false,
    }
}
