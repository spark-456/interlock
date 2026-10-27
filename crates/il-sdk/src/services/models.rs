//! Model gateway (PRD §8.6). Code asks for a role; config maps roles to model IDs.

use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;
use il_core::FileRef;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Which model a caller wants. Never a model ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelRole {
    Architect,
    Agent,
    Fast,
}

/// Chat message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    System,
    User,
    Assistant,
    Tool,
}

/// One chat message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    /// A system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: ChatRole::System, content: content.into(), tool_calls: vec![], tool_call_id: None }
    }
    /// A user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: ChatRole::User, content: content.into(), tool_calls: vec![], tool_call_id: None }
    }
    /// An assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: ChatRole::Assistant, content: content.into(), tool_calls: vec![], tool_call_id: None }
    }
    /// A tool result message.
    pub fn tool(call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: ChatRole::Tool, content: content.into(), tool_calls: vec![], tool_call_id: Some(call_id.into()) }
    }
}

/// A tool the model may call.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolSpec {
    pub name: String,
    pub description: String,
    /// JSON Schema of the arguments.
    pub parameters: Value,
}

/// A tool call requested by the model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

/// Chat request.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub tools: Vec<ToolSpec>,
    /// Ask for JSON matching this schema.
    #[serde(default)]
    pub json_schema: Option<Value>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
}

/// Token and cost accounting for one call.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cost_usd: f64,
}

/// Chat response.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
    #[serde(default)]
    pub usage: Usage,
    /// Model actually used (for logs; never shown to users outside Advanced).
    #[serde(default)]
    pub model: String,
}

impl ChatResponse {
    /// Parses `content` as JSON.
    pub fn json(&self) -> Result<Value, ModelError> {
        serde_json::from_str(&self.content).map_err(|e| ModelError::InvalidOutput(e.to_string()))
    }
}

/// A streamed chunk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChatDelta {
    Text(String),
    ToolCall(ToolCall),
    Done(Usage),
}

/// Streaming response.
pub type ChatStream = Pin<Box<dyn Stream<Item = Result<ChatDelta, ModelError>> + Send>>;

/// Audio transcription result.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Transcript {
    pub text: String,
    #[serde(default)]
    pub segments: Vec<TranscriptSegment>,
    #[serde(default)]
    pub language: Option<String>,
}

/// Timed transcript segment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptSegment {
    pub start_sec: f32,
    pub end_sec: f32,
    pub text: String,
}

/// Model errors.
#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("model provider unavailable: {0}")]
    Unavailable(String),
    #[error("rate limited")]
    RateLimited,
    #[error("invalid model output: {0}")]
    InvalidOutput(String),
    #[error("spend cap reached")]
    CapReached,
    #[error("model config: {0}")]
    Config(String),
    #[error("{0}")]
    Other(String),
}

/// What nodes, the architect and publish call.
#[async_trait]
pub trait ModelGateway: Send + Sync {
    /// Chat completion (tools, JSON schema, temperature).
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError>;
    /// Streaming chat completion.
    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError>;
    /// Embeddings with the configured embedding model.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError>;
    /// Speech to text.
    async fn transcribe(&self, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError>;
}

/// A concrete provider (Token Factory, fake…). The gateway in `il-ai` maps roles → (provider, model id).
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// Provider name as used in config.
    fn name(&self) -> &str;
    /// Chat with an explicit model id.
    async fn chat(&self, model: &str, req: ChatRequest) -> Result<ChatResponse, ModelError>;
    /// Streaming chat with an explicit model id.
    async fn chat_stream(&self, model: &str, req: ChatRequest) -> Result<ChatStream, ModelError>;
    /// Embeddings with an explicit model id.
    async fn embed(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError>;
    /// Model ids the provider reports (startup check).
    async fn list_models(&self) -> Result<Vec<String>, ModelError>;
}
