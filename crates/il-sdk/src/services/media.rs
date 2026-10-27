//! Media generation (video, voice, image). Adapters self-register via `MediaGenFactory`;
//! the `recorded` provider (fixture media) is the default in demo mode, test runs and tests.

use async_trait::async_trait;
use il_core::FileRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Kind of generated media.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Voice,
    Image,
}

/// A generation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaRequest {
    pub kind: MediaKind,
    pub workspace_id: String,
    pub prompt: String,
    /// Kind-specific parameters (duration, aspect, voice, size…).
    #[serde(default)]
    pub params: Value,
    /// Force fixture media for test runs and demo workspaces.
    #[serde(default)]
    pub recorded: bool,
}

/// Generated media.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaResult {
    pub file: FileRef,
    pub cost_usd: f64,
    /// Provider name (`recorded`, …).
    pub provider: String,
}

/// Media errors.
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("no media provider configured for {0:?}")]
    NotConfigured(MediaKind),
    #[error("media provider: {0}")]
    Provider(String),
    #[error("{0}")]
    Other(String),
}

/// A media generator.
#[async_trait]
pub trait MediaGen: Send + Sync {
    /// Generates media for the request.
    async fn generate(&self, req: MediaRequest) -> Result<MediaResult, MediaError>;
}
