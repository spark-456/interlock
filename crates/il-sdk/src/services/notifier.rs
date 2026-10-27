//! Outbound notifications and approval requests (in-app, Slack…). Channel adapters live with
//! their connectors and are selected by `channel`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Notify errors.
#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    #[error("unknown channel: {0}")]
    UnknownChannel(String),
    #[error("send failed: {0}")]
    Send(String),
}

/// A message to a person.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub workspace_id: String,
    /// `inapp`, `slack`, `email`, `discord`, `telegram`.
    pub channel: String,
    /// Channel-specific destination from node settings (never from item data).
    pub target: Option<String>,
    pub title: String,
    pub body: String,
    /// When set, the message carries Approve/Reject actions for this approval.
    pub approval_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
    /// Credential id for channels that need one.
    #[serde(default)]
    pub credential_id: Option<String>,
}

/// Sends notifications.
#[async_trait]
pub trait Notifier: Send + Sync {
    /// Sends and returns an external message reference (if any).
    async fn send(&self, notification: &Notification) -> Result<Option<String>, NotifyError>;
}
