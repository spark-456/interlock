//! `Gateway`: the `ModelGateway` that maps roles to models, retries, moves to a fallback provider when
//! one can't answer, falls back to JSON-in-prompt when a model lacks native tools or JSON-schema output,
//! validates JSON and meters cost.

use std::collections::{HashMap, HashSet};
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

use crate::config::{Fallback, GatewayConfig, Roles, cost, role_name};
use crate::fallback;

/// Repair attempts after a reply that doesn't match the requested JSON schema.
const REPAIRS: usize = 2;

/// One provider with its own model ids and prices.
struct Link {
    provider: Arc<dyn ModelProvider>,
    roles: Roles,
    prices: HashMap<String, [f64; 2]>,
}

impl Link {
    fn meter(&self, role: &str, model: &str, mut usage: Usage) -> Usage {
        usage.cost_usd = cost(&self.prices, role, &usage);
        tracing::debug!(
            provider = self.provider.name(),
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

/// The model gateway used everywhere outside tests.
pub struct Gateway {
    /// The configured provider first, then the fallbacks that started.
    links: Vec<Link>,
    retries: u32,
    backoff_ms: u64,
    /// Models that refused native tools or JSON schema; they get the prompt protocol from then on.
    prompt_only: Mutex<HashSet<String>>,
}

impl Gateway {
    fn empty(config: &GatewayConfig) -> Self {
        Self { links: vec![], retries: config.retries, backoff_ms: config.backoff_ms, prompt_only: Mutex::default() }
    }

    fn push(&mut self, provider: Arc<dyn ModelProvider>, roles: Roles, prices: HashMap<String, [f64; 2]>) {
        self.links.push(Link { provider, roles, prices });
    }

    /// A gateway over an already-built provider.
    pub fn new(provider: Arc<dyn ModelProvider>, config: GatewayConfig) -> Self {
        let mut gateway = Self::empty(&config);
        gateway.push(provider, config.roles, config.prices);
        gateway
    }

    /// Adds `provider` as the next fallback, with the model ids and prices in `fallback`.
    pub fn with_fallback(mut self, provider: Arc<dyn ModelProvider>, fallback: Fallback) -> Self {
        self.push(provider, fallback.roles, fallback.prices);
        self
    }

    /// Builds the configured provider and fallbacks from the registry; `fake` forces the offline provider.
    /// Fallbacks without a key are skipped; so is the primary without its key, when a fallback started.
    pub fn from_config(models: &Value, fake: bool) -> Result<Self, ModelError> {
        let config: GatewayConfig =
            serde_json::from_value(models.clone()).map_err(|e| ModelError::Config(format!("[models]: {e}")))?;
        if fake {
            return Ok(Self::new(build("fake", models)?, config));
        }
        let mut gateway = Self::empty(&config);
        let primary_error = match build(&config.provider, models) {
            Ok(p) => {
                gateway.push(p, config.roles.clone(), config.prices.clone());
                None
            }
            Err(e) => Some(e),
        };
        let sections = models.get("fallbacks").and_then(Value::as_array).cloned().unwrap_or_default();
        for (fallback, section) in config.fallbacks.into_iter().zip(&sections) {
            match build(&fallback.provider, section) {
                Ok(p) => {
                    tracing::info!(provider = fallback.provider, "fallback model provider ready");
                    gateway.push(p, fallback.roles, fallback.prices);
                }
                Err(e) => tracing::info!(provider = fallback.provider, error = %e, "fallback model provider skipped"),
            }
        }
        if let Some(e) = primary_error {
            if gateway.links.is_empty() {
                return Err(e);
            }
            tracing::warn!(provider = config.provider, error = %e, "model provider not started; using the fallbacks");
        }
        Ok(gateway)
    }

    /// Startup check: logs each provider's models and returns the configured ones it doesn't offer.
    pub async fn check(&self) -> Vec<String> {
        let mut missing = Vec::new();
        for link in &self.links {
            let provider = link.provider.name();
            let offered = match link.provider.list_models().await {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(provider, error = %e, "could not list models; skipping the model check");
                    continue;
                }
            };
            tracing::info!(provider, count = offered.len(), "models available");
            for (role, id) in link.roles.all() {
                if offered.iter().any(|m| m == id) {
                    tracing::info!(provider, role, model = id, "model ok");
                } else {
                    tracing::error!(
                        provider,
                        role,
                        model = id,
                        "model not offered by the provider; fix its [models] roles"
                    );
                    missing.push(id.to_string());
                }
            }
        }
        missing
    }

    /// The providers with a model for a job, in order, with that model.
    fn offering<'a>(
        &'a self,
        model: impl Fn(&'a Link) -> Option<&'a str> + 'a,
    ) -> impl Iterator<Item = (&'a Link, &'a str)> + 'a {
        self.links.iter().filter_map(move |l| Some((l, model(l)?)))
    }

    async fn retry<T, F, Fut>(&self, mut call: F) -> Result<T, ModelError>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, ModelError>>,
    {
        let mut delay = self.backoff_ms;
        let mut attempt = 0;
        loop {
            match call().await {
                Err(e @ (ModelError::RateLimited | ModelError::Unavailable(_))) if attempt < self.retries => {
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
    async fn call(
        &self,
        provider: &dyn ModelProvider,
        model: &str,
        req: &ChatRequest,
    ) -> Result<ChatResponse, ModelError> {
        let needs_native = !req.tools.is_empty() || req.json_schema.is_some();
        if !needs_native || !self.prompt_only(model) {
            match self.retry(|| provider.chat(model, req.clone())).await {
                Err(ModelError::Unsupported(why)) if needs_native => {
                    tracing::info!(model, reason = %why, "falling back to JSON-in-prompt");
                    if let Ok(mut s) = self.prompt_only.lock() {
                        s.insert(model.to_string());
                    }
                }
                other => return other,
            }
        }
        let mut resp = self.retry(|| provider.chat(model, fallback::prompt(req))).await?;
        if !req.tools.is_empty() {
            fallback::parse_tool_calls(&mut resp);
        }
        Ok(resp)
    }

    /// Validates a reply against the requested schema, asking the model to repair it when it doesn't match.
    async fn ensure_schema(
        &self,
        provider: &dyn ModelProvider,
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
            resp = self.retry(|| provider.chat(model, repair.clone())).await?;
            usage = add(usage, resp.usage);
        }
        resp.usage = usage;
        Ok(resp)
    }

    async fn chat_on(
        &self,
        link: &Link,
        model: &str,
        role: &str,
        req: &ChatRequest,
    ) -> Result<ChatResponse, ModelError> {
        let resp = self.call(&*link.provider, model, req).await?;
        let mut resp = self.ensure_schema(&*link.provider, model, req, resp).await?;
        resp.usage = link.meter(role, model, resp.usage);
        resp.model = model.to_string();
        Ok(resp)
    }

    async fn stream_on(
        &self,
        link: &Link,
        model: &str,
        role: &'static str,
        req: &ChatRequest,
    ) -> Result<ChatStream, ModelError> {
        if !req.tools.is_empty() && self.prompt_only(model) {
            return Err(ModelError::Unsupported("tools in prompt".into()));
        }
        let stream = self.retry(|| link.provider.chat_stream(model, req.clone())).await?;
        let (prices, provider, model) = (link.prices.clone(), link.provider.name().to_string(), model.to_string());
        Ok(Box::pin(stream.map(move |d| match d {
            Ok(ChatDelta::Done(mut u)) => {
                u.cost_usd = cost(&prices, role, &u);
                tracing::debug!(
                    provider,
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

    /// A structured reply, validated whole and sent as one chunk.
    async fn whole(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let resp = self.chat(role, req).await?;
        let mut chunks: Vec<Result<ChatDelta, ModelError>> = Vec::new();
        if !resp.content.is_empty() {
            chunks.push(Ok(ChatDelta::Text(resp.content)));
        }
        chunks.extend(resp.tool_calls.into_iter().map(|c| Ok(ChatDelta::ToolCall(c))));
        chunks.push(Ok(ChatDelta::Done(resp.usage)));
        Ok(Box::pin(futures::stream::iter(chunks)))
    }
}

#[async_trait]
impl ModelGateway for Gateway {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        let name = role_name(role);
        let mut last = unset(name);
        for (link, model) in self.offering(|l| l.roles.chat(role)) {
            match self.chat_on(link, model, name, &req).await {
                Err(e) if moves_on(link, model, &e) => last = e,
                other => return other,
            }
        }
        Err(last)
    }

    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        // Structured replies are validated whole, so they arrive as one chunk.
        if req.json_schema.is_some() {
            return self.whole(role, req).await;
        }
        let name = role_name(role);
        let mut last = unset(name);
        for (link, model) in self.offering(|l| l.roles.chat(role)) {
            match self.stream_on(link, model, name, &req).await {
                Err(e) if moves_on(link, model, &e) => last = e,
                Err(ModelError::Unsupported(_)) => return self.whole(role, req).await,
                other => return other,
            }
        }
        Err(last)
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        let mut last = unset("embed");
        for (link, model) in self.offering(|l| (!l.roles.embed.is_empty()).then_some(l.roles.embed.as_str())) {
            match self.retry(|| link.provider.embed(model, texts)).await {
                Err(e) if moves_on(link, model, &e) => last = e,
                other => return other,
            }
        }
        Err(last)
    }

    async fn transcribe(&self, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
        let mut last = unset("transcribe");
        for (link, model) in self.offering(|l| l.roles.transcribe.as_deref()) {
            match self.retry(|| link.provider.transcribe(model, file, bytes)).await {
                Err(e) if moves_on(link, model, &e) => last = e,
                other => return other,
            }
        }
        Err(last)
    }
}

fn unset(job: &str) -> ModelError {
    ModelError::Config(format!("set models.roles.{job}"))
}

/// Whether another provider may answer after `e` (limits, outages, keys, credit, unknown models).
fn moves_on(link: &Link, model: &str, e: &ModelError) -> bool {
    let next = matches!(e, ModelError::RateLimited | ModelError::Unavailable(_) | ModelError::Config(_));
    if next {
        tracing::warn!(provider = link.provider.name(), model, error = %e, "trying the next model provider");
    }
    next
}

/// The registered provider `name`, built from its config section.
fn build(name: &str, section: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    let factory = il_sdk::registry::providers()
        .find(|p| p.name == name)
        .ok_or_else(|| ModelError::Config(format!("no model provider named {name}")))?;
    (factory.make)(section)
}

fn add(a: Usage, b: Usage) -> Usage {
    Usage {
        prompt_tokens: a.prompt_tokens + b.prompt_tokens,
        completion_tokens: a.completion_tokens + b.completion_tokens,
        cost_usd: a.cost_usd + b.cost_usd,
    }
}
