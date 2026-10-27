//! `fake` provider (`IL_PROVIDERS=fake`): deterministic, offline. Echoes the last user message.

use std::sync::Arc;

use async_trait::async_trait;
use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{
    ChatDelta, ChatRequest, ChatResponse, ChatRole, ChatStream, ModelError, ModelProvider, Usage,
};
use serde_json::Value;

/// Offline provider used by tests and fake mode.
#[derive(Debug, Default)]
pub struct FakeProvider;

#[async_trait]
impl ModelProvider for FakeProvider {
    fn name(&self) -> &str {
        "fake"
    }

    async fn chat(&self, model: &str, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        let last = req.messages.iter().rev().find(|m| m.role == ChatRole::User);
        Ok(ChatResponse {
            content: last.map(|m| m.content.clone()).unwrap_or_default(),
            usage: Usage::default(),
            model: model.to_string(),
            ..Default::default()
        })
    }

    async fn chat_stream(&self, model: &str, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let resp = self.chat(model, req).await?;
        Ok(Box::pin(futures::stream::iter([Ok(ChatDelta::Text(resp.content)), Ok(ChatDelta::Done(resp.usage))])))
    }

    async fn embed(&self, _model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        Ok(texts.iter().map(|t| vec![t.len() as f32, 1.0]).collect())
    }

    async fn list_models(&self) -> Result<Vec<String>, ModelError> {
        Ok(vec!["fake".into()])
    }
}

fn make(_config: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    Ok(Arc::new(FakeProvider))
}

inventory::submit! { ProviderFactory { name: "fake", make } }

#[cfg(test)]
mod tests {
    use il_sdk::services::models::ChatMessage;

    use super::*;

    #[tokio::test]
    async fn registered_and_echoes() {
        let factory = il_sdk::registry::providers().find(|p| p.name == "fake").expect("fake provider registered");
        let provider = (factory.make)(&Value::Null).unwrap();
        let req = ChatRequest { messages: vec![ChatMessage::user("hello")], ..Default::default() };
        assert_eq!(provider.chat("m", req).await.unwrap().content, "hello");
    }
}
