//! Building flows from goals. Implemented by `il-architect`; used by the server and platform MCP.

use async_trait::async_trait;
use il_core::Flow;
use il_core::dto::{AskDiff, BuildAnswers, BuildRequest};

/// Architect errors.
#[derive(Debug, thiserror::Error)]
pub enum ArchitectError {
    #[error("build not found: {0}")]
    NotFound(String),
    #[error("{0}")]
    Failed(String),
}

/// Architect operations. Builds run in the background and stream `BuildEvent`s to the store.
#[async_trait]
pub trait ArchitectService: Send + Sync {
    /// Starts a build; returns the build id.
    async fn start_build(&self, workspace_id: &str, req: BuildRequest) -> Result<String, ArchitectError>;
    /// Answers clarifying questions.
    async fn answer(&self, build_id: &str, answers: BuildAnswers) -> Result<(), ArchitectError>;
    /// Turns a change request into a JSON-Patch diff against `flow` (AR-8), about the `focus` steps when given.
    async fn ask(&self, flow: &Flow, message: &str, focus: &[String]) -> Result<AskDiff, ArchitectError>;
}
