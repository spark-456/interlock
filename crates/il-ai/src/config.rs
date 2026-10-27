//! The `[models]` section of `interlock.toml`.

use std::collections::HashMap;

use il_sdk::services::models::{ModelError, ModelRole, Usage};
use serde::Deserialize;

/// Provider, role → model id, prices and retry policy.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GatewayConfig {
    /// Registered provider name (`tokenfactory`, `fake`).
    pub provider: String,
    pub base_url: String,
    pub roles: Roles,
    /// USD per 1M tokens `[input, output]`, keyed by role name.
    pub prices: HashMap<String, [f64; 2]>,
    /// Retries after a rate limit or an unavailable provider.
    pub retries: u32,
    /// First backoff; doubles on each retry.
    pub backoff_ms: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            provider: "tokenfactory".into(),
            base_url: "https://api.tokenfactory.nebius.com/v1/".into(),
            roles: Roles::default(),
            prices: HashMap::new(),
            retries: 3,
            backoff_ms: 500,
        }
    }
}

/// Model id per role.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Roles {
    pub architect: String,
    pub agent: String,
    pub fast: String,
    pub embed: String,
    /// Speech-to-text model; transcription is off when unset.
    pub transcribe: Option<String>,
}

impl GatewayConfig {
    /// The model id configured for `role`.
    pub fn model(&self, role: ModelRole) -> Result<&str, ModelError> {
        let id = match role {
            ModelRole::Architect => &self.roles.architect,
            ModelRole::Agent => &self.roles.agent,
            ModelRole::Fast => &self.roles.fast,
        };
        if id.is_empty() {
            return Err(ModelError::Config(format!("set models.roles.{}", role_name(role))));
        }
        Ok(id)
    }

    /// Cost of `usage` at the prices configured for `role` (0 when unpriced).
    pub fn cost(&self, role: &str, usage: &Usage) -> f64 {
        let [input, output] = self.prices.get(role).copied().unwrap_or([0.0, 0.0]);
        (f64::from(usage.prompt_tokens) * input + f64::from(usage.completion_tokens) * output) / 1_000_000.0
    }

    /// Every configured `(role, model id)`, for the startup check.
    pub fn all_models(&self) -> Vec<(&'static str, &str)> {
        let r = &self.roles;
        let mut all =
            vec![("architect", r.architect.as_str()), ("agent", &r.agent), ("fast", &r.fast), ("embed", &r.embed)];
        if let Some(t) = &r.transcribe {
            all.push(("transcribe", t));
        }
        all.retain(|(_, id)| !id.is_empty());
        all
    }
}

/// The config and price key for a role.
pub fn role_name(role: ModelRole) -> &'static str {
    match role {
        ModelRole::Architect => "architect",
        ModelRole::Agent => "agent",
        ModelRole::Fast => "fast",
    }
}
