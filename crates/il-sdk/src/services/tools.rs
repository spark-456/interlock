//! Tools the Agent node can call (OpenAPI operations, knowledge search, nodes).

use async_trait::async_trait;
use serde_json::Value;

/// Tool errors.
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("invalid arguments: {0}")]
    InvalidArguments(String),
    #[error("tool failed: {0}")]
    Failed(String),
}

/// A callable tool with a JSON-schema signature.
#[async_trait]
pub trait AgentTool: Send + Sync {
    /// Unique tool name (`search_help_center`, `get_invoice`).
    fn name(&self) -> &str;
    /// What the tool does (shown to the model).
    fn description(&self) -> &str;
    /// JSON Schema of the arguments.
    fn schema(&self) -> Value;
    /// Calls the tool.
    async fn invoke(&self, args: Value) -> Result<Value, ToolError>;
}
