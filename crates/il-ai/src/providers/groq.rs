//! Groq (OpenAI-compatible). Key from `GROQ_API_KEY`.

use std::sync::Arc;

use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{ModelError, ModelProvider};
use serde_json::Value;

fn make(config: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    super::openai::make("groq", "GROQ_API_KEY", "https://api.groq.com/openai/v1/", config)
}

inventory::submit! { ProviderFactory { name: "groq", make } }
