//! `Gateway`: the `ModelGateway` that maps roles to models on one provider, retries, falls back to
//! JSON-in-prompt when a model lacks native tools or JSON-schema output, validates JSON and meters cost.

use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use il_core::FileRef;
use il_sdk::services::models::{
    ChatDelta, ChatMessage, ChatRequest, ChatResponse, ChatStream, ModelError, ModelGateway, ModelProvider, ModelRole,
    Transcript, Usage,
};
use serde_json::Value;

use crate::config::{GatewayConfig, role_name};
use crate::fallback;

/// Repair attempts after a reply that doesn't match the requested JSON schema.
const REPAIRS: usize = 2;

/// The model gateway used everywhere outside tests.
pub struct Gateway {
    provider: Arc<dyn ModelProvider>,
    config: GatewayConfig,
    /// Models that refused native tools or JSON schema; they get the prompt protocol from then on.
    prompt_only: Mutex<HashSet<String>>,
}

impl Gateway {
    /// A gateway over an already-built provider.
    pub fn new(provider: Arc<dyn ModelProvider>, config: GatewayConfig) -> Self {
        Self { provider, config, prompt_only: Mutex::new(HashSet::new()) }
    }

    /// Builds the configured provider from the registry; `fake` forces the offline provider.
    pub fn from_config(models: &Value, fake: bool) -> Result<Self, ModelError> {
        let config: GatewayConfig =
            serde_json::from_value(models.clone()).map_err(|e| ModelError::Config(format!("[models]: {e}")))?;
        let name = if fake { "fake" } else { config.provider.as_str() };
        let factory = il_sdk::registry::providers()
            .find(|p| p.name == name)
            .ok_or_else(|| ModelError::Config(format!("no model provider named {name}")))?;
        Ok(Self::new((factory.make)(models)?, config))
    }

    /// Startup check: logs the provider's models and returns the configured ones it doesn't offer.
    pub async fn check(&self) -> Vec<String> {
        let offered = match self.provider.list_models().await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(provider = self.provider.name(), error = %e, "could not list models; skipping the model check");
                return vec![];
            }
        };
        tracing::info!(provider = self.provider.name(), count = offered.len(), "models available");
        let mut missing = Vec::new();
        for (role, id) in self.config.all_models() {
            if offered.iter().any(|m| m == id) {
                tracing::info!(role, model = id, "model ok");
            } else {
                tracing::error!(role, model = id, "model not offered by the provider; fix models.roles.{role}");
                missing.push(id.to_string());
            }
        }
        missing
    }

    async fn retry<T, F, Fut>(&self, mut call: F) -> Result<T, ModelError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ModelError>>,
    {
        let mut delay = self.config.backoff_ms;
        let mut attempt = 0;
        loop {
            match call().await {
                Err(e @ (ModelError::RateLimited | ModelError::Unavailable(_))) if attempt < self.config.retries => {
                    tracing::warn!(error = %e, attempt, "model call failed; retrying");
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    delay *= 2;
                    attempt += 1;
                }
                other => return other,
            }
        }
    }

    fn prompt_only(&self, model: &str) -> bool {
        self.prompt_only.lock().is_ok_and(|s| s.contains(model))
    }

    /// One call, native when the model supports it, otherwise through the prompt protocol.
    async fn call(&self, model: &str, req: &ChatRequest) -> Result<ChatResponse, ModelError> {
        let needs_native = !req.tools.is_empty() || req.json_schema.is_some();
        if !needs_native || !self.prompt_only(model) {
            match self.retry(|| self.provider.chat(model, req.clone())).await {
                Err(ModelError::Unsupported(why)) if needs_native => {
                    tracing::info!(model, reason = %why, "falling back to JSON-in-prompt");
                    if let Ok(mut s) = self.prompt_only.lock() {
                        s.insert(model.to_string());
                    }
                }
                other => return other,
            }
        }
        let mut resp = self.retry(|| self.provider.chat(model, fallback::prompt(req))).await?;
        if !req.tools.is_empty() {
            fallback::parse_tool_calls(&mut resp);
        }
        Ok(resp)
    }

    /// Validates a reply against the requested schema, asking the model to repair it when it doesn't match.
    async fn ensure_schema(
        &self,
        model: &str,
        req: &ChatRequest,
        mut resp: ChatResponse,
    ) -> Result<ChatResponse, ModelError> {
        let Some(schema) = &req.json_schema else { return Ok(resp) };
        let mut usage = resp.usage;
        for attempt in 0..=REPAIRS {
            if !resp.tool_calls.is_empty() {
                break;
            }
            let problem = match fallback::extract_json(&resp.content) {
                Some(v) => match il_sdk::validate::validate_json(schema, &v) {
                    Ok(()) => {
                        resp.content = v.to_string();
                        break;
                    }
                    Err(issues) => {
                        issues.iter().map(|i| format!("{} {}", i.path, i.message)).collect::<Vec<_>>().join("; ")
                    }
                },
                None => "it is not JSON".to_string(),
            };
            if attempt == REPAIRS {
                return Err(ModelError::InvalidOutput(format!("reply does not match the schema: {problem}")));
            }
            let mut repair = fallback::prompt(&ChatRequest { tools: vec![], ..req.clone() });
            repair.messages.push(ChatMessage::assistant(resp.content.clone()));
            repair.messages.push(ChatMessage::user(format!(
                "That reply is invalid ({problem}). Reply again with only JSON matching the schema."
            )));
            resp = self.retry(|| self.provider.chat(model, repair.clone())).await?;
            usage = add(usage, resp.usage);
        }
        resp.usage = usage;
        Ok(resp)
    }

    fn meter(&self, role: &str, model: &str, mut usage: Usage) -> Usage {
        usage.cost_usd = self.config.cost(role, &usage);
        tracing::debug!(
            role,
            model,
            prompt = usage.prompt_tokens,
            completion = usage.completion_tokens,
            cost_usd = usage.cost_usd,
            "model call"
        );
        usage
    }
}

#[async_trait]
impl ModelGateway for Gateway {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        let model = self.config.model(role)?;
        let resp = self.call(model, &req).await?;
        let mut resp = self.ensure_schema(model, &req, resp).await?;
        resp.usage = self.meter(role_name(role), model, resp.usage);
        resp.model = model.to_string();
        Ok(resp)
    }

    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let model = self.config.model(role)?;
        // Structured replies are validated whole, so they arrive as one chunk.
        let stream = if req.json_schema.is_some() || (!req.tools.is_empty() && self.prompt_only(model)) {
            Err(ModelError::Unsupported("structured reply".into()))
        } else {
            self.retry(|| self.provider.chat_stream(model, req.clone())).await
        };
        let stream = match stream {
            Err(ModelError::Unsupported(_)) => {
                let resp = self.chat(role, req).await?;
                let mut chunks: Vec<Result<ChatDelta, ModelError>> = Vec::new();
                if !resp.content.is_empty() {
                    chunks.push(Ok(ChatDelta::Text(resp.content)));
                }
                chunks.extend(resp.tool_calls.into_iter().map(|c| Ok(ChatDelta::ToolCall(c))));
                chunks.push(Ok(ChatDelta::Done(resp.usage)));
                return Ok(Box::pin(futures::stream::iter(chunks)));
            }
            other => other?,
        };
        let config = self.config.clone();
        let (role, model) = (role_name(role), model.to_string());
        Ok(Box::pin(stream.map(move |d| match d {
            Ok(ChatDelta::Done(mut u)) => {
                u.cost_usd = config.cost(role, &u);
                tracing::debug!(
                    role,
                    model,
                    prompt = u.prompt_tokens,
                    completion = u.completion_tokens,
                    cost_usd = u.cost_usd,
                    "model stream"
                );
                Ok(ChatDelta::Done(u))
            }
            other => other,
        })))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        let model = &self.config.roles.embed;
        if model.is_empty() {
            return Err(ModelError::Config("set models.roles.embed".into()));
        }
        self.retry(|| self.provider.embed(model, texts)).await
    }

    async fn transcribe(&self, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
        let model = self
            .config
            .roles
            .transcribe
            .as_deref()
            .ok_or_else(|| ModelError::Config("set models.roles.transcribe".into()))?;
        self.retry(|| self.provider.transcribe(model, file, bytes)).await
    }
}

fn add(a: Usage, b: Usage) -> Usage {
    Usage {
        prompt_tokens: a.prompt_tokens + b.prompt_tokens,
        completion_tokens: a.completion_tokens + b.completion_tokens,
        cost_usd: a.cost_usd + b.cost_usd,
    }
}
