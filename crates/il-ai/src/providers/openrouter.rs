//! OpenRouter (OpenAI-compatible). Key from `OPENROUTER_API_KEY`.

use std::sync::Arc;

use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{ModelError, ModelProvider};
use serde_json::Value;

fn make(config: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    super::openai::make("openrouter", "OPENROUTER_API_KEY", "https://openrouter.ai/api/v1/", config)
}

inventory::submit! { ProviderFactory { name: "openrouter", make } }
