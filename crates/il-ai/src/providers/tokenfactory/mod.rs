//! Nebius Token Factory (OpenAI-compatible): chat, streaming, tools, JSON schema, embeddings,
//! model list and transcription. Key from `NEBIUS_API_KEY`.

mod stream;
mod wire;

use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use il_core::FileRef;
use il_sdk::SafeHttp;
use il_sdk::registry::ProviderFactory;
use il_sdk::services::models::{
    ChatRequest, ChatResponse, ChatStream, ModelError, ModelProvider, Transcript, TranscriptSegment,
};
use serde_json::{Value, json};

/// The Token Factory client.
pub struct TokenFactory {
    http: SafeHttp,
    base_url: String,
    api_key: String,
}

impl TokenFactory {
    /// A client for `base_url` (e.g. `https://api.tokenfactory.nebius.com/v1/`).
    pub fn new(http: SafeHttp, base_url: &str, api_key: String) -> Self {
        Self { http, base_url: base_url.trim_end_matches('/').to_string(), api_key }
    }

    async fn send(&self, req: reqwest::RequestBuilder) -> Result<reqwest::Response, ModelError> {
        let resp = req.bearer_auth(&self.api_key).send().await.map_err(|e| ModelError::Unavailable(e.to_string()))?;
        if resp.status().is_success() {
            return Ok(resp);
        }
        let status = resp.status().as_u16();
        Err(wire::error(status, &resp.text().await.unwrap_or_default()))
    }

    async fn post(&self, path: &str, body: &Value) -> Result<Value, ModelError> {
        let req = self.http.post(&self.url(path)).map_err(|e| ModelError::Config(e.to_string()))?.json(body);
        self.send(req).await?.json().await.map_err(|e| ModelError::InvalidOutput(e.to_string()))
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{path}", self.base_url)
    }
}

#[async_trait]
impl ModelProvider for TokenFactory {
    fn name(&self) -> &str {
        "tokenfactory"
    }

    async fn chat(&self, model: &str, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        wire::chat_response(&self.post("chat/completions", &wire::chat_body(model, &req, false)).await?)
    }

    async fn chat_stream(&self, model: &str, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let http = self.http.post(&self.url("chat/completions")).map_err(|e| ModelError::Config(e.to_string()))?;
        let resp = self.send(http.json(&wire::chat_body(model, &req, true))).await?;
        Ok(stream::parse(resp.bytes_stream().boxed()))
    }

    async fn embed(&self, model: &str, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        let v = self.post("embeddings", &json!({"model": model, "input": texts})).await?;
        let mut data: Vec<&Value> =
            v.get("data").and_then(Value::as_array).map(|d| d.iter().collect()).unwrap_or_default();
        data.sort_by_key(|d| d.get("index").and_then(Value::as_u64).unwrap_or(0));
        let vectors: Vec<Vec<f32>> = data
            .iter()
            .map(|d| {
                let nums = d.get("embedding").and_then(Value::as_array).into_iter().flatten();
                nums.filter_map(Value::as_f64).map(|x| x as f32).collect()
            })
            .collect();
        if vectors.len() != texts.len() {
            return Err(ModelError::InvalidOutput(format!("{} embeddings for {} texts", vectors.len(), texts.len())));
        }
        Ok(vectors)
    }

    async fn list_models(&self) -> Result<Vec<String>, ModelError> {
        let req = self.http.get(&self.url("models")).map_err(|e| ModelError::Config(e.to_string()))?;
        let v: Value = self.send(req).await?.json().await.map_err(|e| ModelError::InvalidOutput(e.to_string()))?;
        let ids = v.get("data").and_then(Value::as_array).into_iter().flatten();
        Ok(ids.filter_map(|m| m.get("id").and_then(Value::as_str).map(String::from)).collect())
    }

    async fn transcribe(&self, model: &str, file: &FileRef, bytes: &[u8]) -> Result<Transcript, ModelError> {
        let part = reqwest::multipart::Part::bytes(bytes.to_vec())
            .file_name(file.name.clone())
            .mime_str(&file.mime)
            .map_err(|e| ModelError::Other(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("model", model.to_string())
            .text("response_format", "verbose_json")
            .part("file", part);
        let req = self.http.post(&self.url("audio/transcriptions")).map_err(|e| ModelError::Config(e.to_string()))?;
        let v: Value =
            self.send(req.multipart(form)).await?.json().await.map_err(|e| ModelError::InvalidOutput(e.to_string()))?;
        let segments = v.get("segments").and_then(Value::as_array).into_iter().flatten();
        Ok(Transcript {
            text: v.get("text").and_then(Value::as_str).unwrap_or_default().to_string(),
            segments: segments
                .map(|s| TranscriptSegment {
                    start_sec: s.get("start").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                    end_sec: s.get("end").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                    text: s.get("text").and_then(Value::as_str).unwrap_or_default().trim().to_string(),
                })
                .collect(),
            language: v.get("language").and_then(Value::as_str).map(String::from),
        })
    }
}

fn make(config: &Value) -> Result<Arc<dyn ModelProvider>, ModelError> {
    let api_key = std::env::var("NEBIUS_API_KEY").map_err(|_| ModelError::Config("set NEBIUS_API_KEY".into()))?;
    let base_url = config.get("base_url").and_then(Value::as_str).unwrap_or("https://api.tokenfactory.nebius.com/v1/");
    let http = SafeHttp::new().map_err(|e| ModelError::Config(e.to_string()))?;
    Ok(Arc::new(TokenFactory::new(http, base_url, api_key)))
}

inventory::submit! { ProviderFactory { name: "tokenfactory", make } }
