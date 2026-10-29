//! `fake` provider (`IL_PROVIDERS=fake`): deterministic, offline. Echoes the last user message, or,
//! when the request asks for JSON, returns a minimal instance of its schema so structured steps run.
//! Offered a knowledge search, it searches once for the question and answers with the best passage.
//! Transcribes any audio to a line naming the file.

use std::sync::Arc;

use async_trait::async_trait;
use il_core::FileRef;
use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{
    ChatDelta, ChatRequest, ChatResponse, ChatRole, ChatStream, ModelError, ModelProvider, ToolCall, Transcript, Usage,
};
use serde_json::{Value, json};

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
        let echo = last.map(|m| m.content.clone()).unwrap_or_default();
        let searched = req.messages.iter().rev().find(|m| m.role == ChatRole::Tool);
        if let (Some(search), None) = (req.tools.iter().find(|t| t.name.starts_with("search_")), searched) {
            let call = ToolCall { id: "call_0".into(), name: search.name.clone(), arguments: json!({ "query": echo }) };
            return Ok(ChatResponse { tool_calls: vec![call], model: model.to_string(), ..Default::default() });
        }
        let passage = searched
            .and_then(|m| serde_json::from_str::<Value>(&m.content).ok())
            .and_then(|hits| hits[0]["text"].as_str().map(str::to_string));
        let content = match (&req.json_schema, passage) {
            (Some(schema), _) => sample(schema, &echo).to_string(),
            (None, Some(passage)) => passage,
            (None, None) => echo,
        };
        Ok(ChatResponse { content, usage: Usage::default(), model: model.to_string(), ..Default::default() })
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

    async fn transcribe(&self, _model: &str, file: &FileRef, _bytes: &[u8]) -> Result<Transcript, ModelError> {
        Ok(Transcript { text: format!("Transcript of {}", file.name), ..Default::default() })
    }
}

/// The smallest value matching `schema`: first enum value or option, every property, one array
/// item when one is required, and `text` for free strings.
fn sample(schema: &Value, text: &str) -> Value {
    if let Some(first) = schema["enum"].as_array().and_then(|e| e.first()) {
        return first.clone();
    }
    if let Some(first) = schema["oneOf"].as_array().or(schema["anyOf"].as_array()).and_then(|o| o.first()) {
        return sample(first, text);
    }
    match schema["type"].as_str() {
        Some("object") => Value::Object(
            schema["properties"]
                .as_object()
                .map(|props| props.iter().map(|(k, v)| (k.clone(), sample(v, text))).collect())
                .unwrap_or_default(),
        ),
        Some("array") if schema["minItems"].as_u64().unwrap_or(0) > 0 => {
            Value::Array(vec![sample(&schema["items"], text)])
        }
        Some("array") => Value::Array(Vec::new()),
        Some("boolean") => Value::Bool(true),
        Some("number" | "integer") => schema["minimum"].clone().as_f64().map_or(Value::from(0), Value::from),
        _ => Value::String(text.into()),
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

    #[tokio::test]
    async fn answers_json_requests_with_a_schema_instance() {
        let schema = serde_json::json!({"type": "object", "properties": {
            "label": {"type": "string", "enum": ["billing", "support"]},
            "answer": {"type": "string"},
            "confident": {"type": "boolean"},
            "tags": {"type": "array", "items": {"type": "string"}},
            "steps": {"type": "array", "minItems": 1, "items": {"type": "integer", "minimum": 2}}}});
        let req = ChatRequest {
            messages: vec![ChatMessage::user("refund?")],
            json_schema: Some(schema),
            ..Default::default()
        };
        let out: Value = serde_json::from_str(&FakeProvider.chat("m", req).await.unwrap().content).unwrap();
        assert_eq!(
            out,
            serde_json::json!({"label": "billing", "answer": "refund?", "confident": true, "tags": [], "steps": [2.0]})
        );
    }
}
