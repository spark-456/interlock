//! The `[models]` section of `interlock.toml`.

use std::collections::HashMap;

use il_sdk::services::models::{ModelRole, Usage};
use serde::Deserialize;

/// Provider, role → model id, prices, retry policy and fallback providers.
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
    /// Providers tried in order when the ones before can't answer (rate limit, outage, key, credit).
    pub fallbacks: Vec<Fallback>,
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
            fallbacks: vec![],
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

/// A fallback provider with its own model ids and prices.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Fallback {
    /// Registered provider name (`groq`, `openrouter`).
    pub provider: String,
    /// Empty for the provider's default.
    pub base_url: String,
    /// Roles left empty are never sent to this provider.
    pub roles: Roles,
    /// USD per 1M tokens `[input, output]`, keyed by role name.
    pub prices: HashMap<String, [f64; 2]>,
}

impl Roles {
    /// The chat model set for `role`.
    pub fn chat(&self, role: ModelRole) -> Option<&str> {
        let id = match role {
            ModelRole::Architect => &self.architect,
            ModelRole::Agent => &self.agent,
            ModelRole::Fast => &self.fast,
        };
        (!id.is_empty()).then_some(id.as_str())
    }

    /// Every configured `(role, model id)`, for the startup check.
    pub fn all(&self) -> Vec<(&'static str, &str)> {
        let mut all = vec![
            ("architect", self.architect.as_str()),
            ("agent", &self.agent),
            ("fast", &self.fast),
            ("embed", &self.embed),
        ];
        if let Some(t) = &self.transcribe {
            all.push(("transcribe", t));
        }
        all.retain(|(_, id)| !id.is_empty());
        all
    }
}

/// Cost of `usage` at the `prices` for `role` (0 when unpriced).
pub fn cost(prices: &HashMap<String, [f64; 2]>, role: &str, usage: &Usage) -> f64 {
    let [input, output] = prices.get(role).copied().unwrap_or([0.0, 0.0]);
    (f64::from(usage.prompt_tokens) * input + f64::from(usage.completion_tokens) * output) / 1_000_000.0
}

/// The config and price key for a role.
pub fn role_name(role: ModelRole) -> &'static str {
    match role {
        ModelRole::Architect => "architect",
        ModelRole::Agent => "agent",
        ModelRole::Fast => "fast",
    }
}
