//! Error shape (PRD §8.5): `{ "error": { "code": "snake_case", "message": "Human sentence.", "details": {} } }`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Json;

/// The body of every API error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ApiError {
    pub error: ApiErrorBody,
}

/// Inner error object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct ApiErrorBody {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub details: Json,
}

impl ApiError {
    /// Builds an error with empty details.
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            error: ApiErrorBody {
                code: code.into(),
                message: message.into(),
                details: Json::Object(Default::default()),
            },
        }
    }
}

/// Errors raised by pure core logic.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("expression error: {0}")]
    Expression(String),
    #[error("invalid flow: {0}")]
    InvalidFlow(String),
}
