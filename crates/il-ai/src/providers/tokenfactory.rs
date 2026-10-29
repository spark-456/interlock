//! Nebius Token Factory (OpenAI-compatible). Key from `NEBIUS_API_KEY`.

use std::sync::Arc;

use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{ModelError, ModelProvider};
use serde_json::Value;

fn make(config: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    super::openai::make("tokenfactory", "NEBIUS_API_KEY", "https://api.tokenfactory.nebius.com/v1/", config)
}

inventory::submit! { ProviderFactory { name: "tokenfactory", make } }
