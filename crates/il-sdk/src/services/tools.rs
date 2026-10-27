//! Tools the Agent node can call (OpenAPI operations, knowledge search, nodes).

use async_trait::async_trait;
use il_core::Flow;
use serde_json::Value;
use std::sync::Arc;

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

/// Resolves callable tools for one flow in one workspace.
#[async_trait]
pub trait FlowToolProvider: Send + Sync {
    /// Returns only tools authorized for this workspace and flow.
    async fn for_flow(&self, workspace_id: &str, flow: &Flow) -> Result<Vec<Arc<dyn AgentTool>>, ToolError>;
}

/// Empty tool provider used until an implementation is wired.
pub struct NoFlowTools;

#[async_trait]
impl FlowToolProvider for NoFlowTools {
    async fn for_flow(&self, _workspace_id: &str, _flow: &Flow) -> Result<Vec<Arc<dyn AgentTool>>, ToolError> {
        Ok(Vec::new())
    }
}
