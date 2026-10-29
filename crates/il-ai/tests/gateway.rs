//! Gateway behaviour against a scripted provider: roles, cost, retries, fallback, schema repair, startup check.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::StreamExt;
use il_ai::{Fallback, Gateway, GatewayConfig, Roles};
use il_sdk::services::models::{
    ChatDelta, ChatMessage, ChatRequest, ChatResponse, ChatStream, ModelError, ModelGateway, ModelProvider, ModelRole,
    ToolSpec, Usage,
};
use serde_json::json;

/// Replays queued results and records every request it gets.
#[derive(Default)]
struct Scripted {
    replies: Mutex<VecDeque<Result<ChatResponse, ModelError>>>,
    seen: Mutex<Vec<(String, ChatRequest)>>,
}

impl Scripted {
    fn push(&self, r: Result<ChatResponse, ModelError>) -> &Self {
        self.replies.lock().unwrap().push_back(r);
        self
    }
    fn text(&self, content: &str) -> &Self {
        let usage = Usage { prompt_tokens: 1000, completion_tokens: 500, cost_usd: 0.0 };
        self.push(Ok(ChatResponse { content: content.into(), usage, ..Default::default() }))
    }
    fn seen(&self) -> Vec<(String, ChatRequest)> {
        self.seen.lock().unwrap().clone()
    }
}

#[async_trait]
impl ModelProvider for Scripted {
    fn name(&self) -> &str {
        "scripted"
    }
    async fn chat(&self, model: &str, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        self.seen.lock().unwrap().push((model.into(), req));
        self.replies.lock().unwrap().pop_front().unwrap_or_else(|| Err(ModelError::Other("script ran out".into())))
    }
    async fn chat_stream(&self, model: &str, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let resp = self.chat(model, req).await?;
        Ok(Box::pin(futures::stream::iter([Ok(ChatDelta::Text(resp.content)), Ok(ChatDelta::Done(resp.usage))])))
    }
    async fn embed(&self, _model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        Ok(texts.iter().map(|_| vec![0.0]).collect())
    }
    async fn list_models(&self) -> Result<Vec<String>, ModelError> {
        Ok(vec!["m-arch".into(), "m-agent".into(), "m-fast".into()])
    }
}

fn gateway(provider: &Arc<Scripted>) -> Gateway {
    let config = GatewayConfig {
        roles: Roles {
            architect: "m-arch".into(),
            agent: "m-agent".into(),
            fast: "m-fast".into(),
            embed: "m-embed".into(),
            transcribe: None,
        },
        prices: HashMap::from([("fast".to_string(), [0.06, 0.24])]),
        backoff_ms: 1,
        ..Default::default()
    };
    Gateway::new(provider.clone(), config)
}

fn ask(text: &str) -> ChatRequest {
    ChatRequest { messages: vec![ChatMessage::user(text)], ..Default::default() }
}

#[tokio::test]
async fn maps_roles_to_models_and_meters_cost() {
    let p = Arc::new(Scripted::default());
    p.text("hi");
    let resp = gateway(&p).chat(ModelRole::Fast, ask("hello")).await.unwrap();
    assert_eq!(p.seen()[0].0, "m-fast");
    assert_eq!(resp.model, "m-fast");
    // 1000 × 0.06 + 500 × 0.24 per million tokens.
    assert!((resp.usage.cost_usd - 0.00018).abs() < 1e-12);
}

#[tokio::test]
async fn retries_rate_limits_then_gives_up() {
    let p = Arc::new(Scripted::default());
    p.push(Err(ModelError::RateLimited)).push(Err(ModelError::Unavailable("503".into()))).text("ok");
    assert_eq!(gateway(&p).chat(ModelRole::Agent, ask("x")).await.unwrap().content, "ok");

    let p = Arc::new(Scripted::default());
    for _ in 0..4 {
        p.push(Err(ModelError::RateLimited));
    }
    assert!(matches!(gateway(&p).chat(ModelRole::Agent, ask("x")).await, Err(ModelError::RateLimited)));
    assert_eq!(p.seen().len(), 4, "one call plus three retries");
}

#[tokio::test]
async fn falls_back_to_tools_in_prompt_and_remembers() {
    let p = Arc::new(Scripted::default());
    p.push(Err(ModelError::Unsupported("tools not supported".into())))
        .text(r#"{"tool_calls": [{"name": "get_invoice", "arguments": {"id": 7}}]}"#)
        .text(r#"{"tool_calls": [{"name": "get_invoice", "arguments": {"id": 8}}]}"#);
    let gw = gateway(&p);
    let tool = ToolSpec { name: "get_invoice".into(), description: "Look up an invoice".into(), parameters: json!({}) };
    let req = ChatRequest { tools: vec![tool], ..ask("invoice 7?") };

    let resp = gw.chat(ModelRole::Agent, req.clone()).await.unwrap();
    assert_eq!(resp.tool_calls[0].arguments, json!({"id": 7}));
    let seen = p.seen();
    assert_eq!(seen[0].1.tools.len(), 1, "native first");
    assert!(seen[1].1.tools.is_empty() && seen[1].1.messages[0].content.contains("get_invoice"));

    gw.chat(ModelRole::Agent, req).await.unwrap();
    assert_eq!(p.seen().len(), 3, "the second call goes straight to the prompt protocol");
}

#[tokio::test]
async fn repairs_json_that_misses_the_schema() {
    let p = Arc::new(Scripted::default());
    p.text("Here you go: {\"name\": 3}").text("```json\n{\"name\": \"Larkspur\"}\n```");
    let schema = json!({"type": "object", "required": ["name"], "properties": {"name": {"type": "string"}}});
    let req = ChatRequest { json_schema: Some(schema.clone()), ..ask("name?") };
    let resp = gateway(&p).chat(ModelRole::Fast, req).await.unwrap();
    assert_eq!(resp.json().unwrap(), json!({"name": "Larkspur"}));
    assert_eq!(resp.usage.prompt_tokens, 2000, "usage covers the repair call");
    assert!(p.seen()[1].1.messages.last().unwrap().content.contains("/name expected string"));

    let p = Arc::new(Scripted::default());
    p.text("nope").text("still nope").text("no");
    let req = ChatRequest { json_schema: Some(schema), ..ask("name?") };
    assert!(matches!(gateway(&p).chat(ModelRole::Fast, req).await, Err(ModelError::InvalidOutput(_))));
}

#[tokio::test]
async fn streams_with_cost_on_done() {
    let p = Arc::new(Scripted::default());
    p.text("streamed");
    let chunks: Vec<_> = gateway(&p).chat_stream(ModelRole::Fast, ask("x")).await.unwrap().collect().await;
    assert!(matches!(&chunks[0], Ok(ChatDelta::Text(t)) if t == "streamed"));
    assert!(matches!(&chunks[1], Ok(ChatDelta::Done(u)) if u.cost_usd > 0.0));
}

#[tokio::test]
async fn startup_check_reports_missing_models() {
    let p = Arc::new(Scripted::default());
    assert_eq!(gateway(&p).check().await, vec!["m-embed".to_string()]);
}

#[tokio::test]
async fn transcription_needs_a_model() {
    let p = Arc::new(Scripted::default());
    let file = il_core::FileRef {
        id: "f".into(),
        mime: "audio/mpeg".into(),
        bytes: 0,
        name: "a.mp3".into(),
        preview_url: None,
    };
    assert!(matches!(gateway(&p).transcribe(&file, b"").await, Err(ModelError::Config(_))));
}

fn with_fallback(primary: &Arc<Scripted>, backup: &Arc<Scripted>, roles: Roles) -> Gateway {
    let prices = HashMap::from([("agent".to_string(), [1.0, 1.0])]);
    gateway(primary).with_fallback(backup.clone(), Fallback { roles, prices, ..Default::default() })
}

fn agent_only() -> Roles {
    Roles { agent: "b-agent".into(), ..Default::default() }
}

#[tokio::test]
async fn moves_to_the_fallback_when_the_provider_cannot_answer() {
    let (p, b) = (Arc::new(Scripted::default()), Arc::new(Scripted::default()));
    for _ in 0..4 {
        p.push(Err(ModelError::RateLimited));
    }
    b.text("from backup");
    let resp = with_fallback(&p, &b, agent_only()).chat(ModelRole::Agent, ask("x")).await.unwrap();
    assert_eq!((resp.content.as_str(), resp.model.as_str()), ("from backup", "b-agent"));
    assert!((resp.usage.cost_usd - 0.0015).abs() < 1e-12, "the fallback's own prices");

    let (p, b) = (Arc::new(Scripted::default()), Arc::new(Scripted::default()));
    p.push(Err(ModelError::Config("Token Factory rejected the API key".into())));
    b.text("ok");
    assert_eq!(with_fallback(&p, &b, agent_only()).chat(ModelRole::Agent, ask("x")).await.unwrap().content, "ok");
}

#[tokio::test]
async fn fallbacks_only_take_their_roles_and_retryable_errors() {
    let (p, b) = (Arc::new(Scripted::default()), Arc::new(Scripted::default()));
    p.push(Err(ModelError::Config("no key".into())));
    let gw = with_fallback(&p, &b, agent_only());
    assert!(matches!(gw.chat(ModelRole::Fast, ask("x")).await, Err(ModelError::Config(_))));
    assert!(b.seen().is_empty(), "the fallback has no fast model");

    let (p, b) = (Arc::new(Scripted::default()), Arc::new(Scripted::default()));
    p.push(Err(ModelError::Other("HTTP 400: bad request".into())));
    let gw = with_fallback(&p, &b, agent_only());
    assert!(matches!(gw.chat(ModelRole::Agent, ask("x")).await, Err(ModelError::Other(_))));
    assert!(b.seen().is_empty(), "a bad request would fail anywhere");
}

#[tokio::test]
async fn streams_from_the_fallback() {
    let (p, b) = (Arc::new(Scripted::default()), Arc::new(Scripted::default()));
    for _ in 0..4 {
        p.push(Err(ModelError::Unavailable("out of credit".into())));
    }
    b.text("streamed");
    let chunks: Vec<_> =
        with_fallback(&p, &b, agent_only()).chat_stream(ModelRole::Agent, ask("x")).await.unwrap().collect().await;
    assert!(matches!(&chunks[0], Ok(ChatDelta::Text(t)) if t == "streamed"));
}

#[test]
fn starts_on_a_fallback_when_the_provider_cannot_start() {
    let models = json!({
        "provider": "missing",
        "roles": {"agent": "a"},
        "fallbacks": [{"provider": "also-missing"}, {"provider": "fake", "roles": {"agent": "f"}}]
    });
    assert!(Gateway::from_config(&models, false).is_ok());
    let alone = json!({"provider": "missing", "fallbacks": [{"provider": "also-missing"}]});
    assert!(matches!(Gateway::from_config(&alone, false), Err(ModelError::Config(_))));
}
