//! Knowledge bases (KN-*). Implemented by `il-knowledge`.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Knowledge errors.
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeError {
    #[error("knowledge base not found: {0}")]
    NotFound(String),
    #[error("ingest failed: {0}")]
    Ingest(String),
    #[error("{0}")]
    Other(String),
}

/// A search result with its source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub text: String,
    pub score: f32,
    pub source_id: String,
    pub title: String,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// A source to ingest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SourceInput {
    File { file_id: String, name: String },
    MarkedFile { file_id: String, name: String, role: String },
    Url { url: String, crawl_limit: u32 },
    Text { title: String, text: String },
    InstructionsText { title: String, text: String },
}

/// Knowledge base operations.
#[async_trait]
pub trait KnowledgeService: Send + Sync {
    /// Confirms that a workspace owns a base before externally exposed writes.
    async fn owns_base(&self, _workspace_id: &str, _kb_id: &str) -> Result<bool, KnowledgeError> {
        Ok(false)
    }
    /// Creates a knowledge base and returns its id.
    async fn create_base(&self, workspace_id: &str, name: &str) -> Result<String, KnowledgeError>;
    /// Ingests sources (parse, chunk, embed). Returns the new source ids.
    async fn add_sources(&self, kb_id: &str, sources: Vec<SourceInput>) -> Result<Vec<String>, KnowledgeError>;
    /// Hybrid search (vector + keyword, fused).
    async fn search(&self, kb_id: &str, query: &str, top_k: usize) -> Result<Vec<SearchHit>, KnowledgeError>;
    /// Parses a document to text without storing it (used by `data.parse_document`).
    async fn parse(&self, file_id: &str, format: Option<&str>) -> Result<Value, KnowledgeError>;
}
