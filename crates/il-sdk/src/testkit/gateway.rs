//! Scripted fake `ModelGateway`: queued responses in order; echoes the last user message when empty.

use std::collections::VecDeque;
use std::sync::Mutex;

use async_trait::async_trait;
use il_core::FileRef;
use serde_json::Value;

use crate::services::models::{
    ChatDelta, ChatRequest, ChatResponse, ChatRole, ChatStream, ModelError, ModelGateway, ModelRole, ToolCall,
    Transcript, Usage,
};

/// A fake model gateway with scripted responses.
#[derive(Default)]
pub struct FakeGateway {
    script: Mutex<VecDeque<ChatResponse>>,
    calls: Mutex<Vec<(ModelRole, ChatRequest)>>,
    transcript: Mutex<Option<String>>,
}

impl FakeGateway {
    /// Queues a text response.
    pub fn push_text(&self, text: impl Into<String>) -> &Self {
        self.push(ChatResponse { content: text.into(), model: "fake".into(), ..Default::default() })
    }
    /// Queues a JSON response.
    pub fn push_json(&self, value: Value) -> &Self {
        self.push_text(value.to_string())
    }
    /// Queues a tool call.
    pub fn push_tool_call(&self, name: &str, arguments: Value) -> &Self {
        let call = ToolCall { id: crate::id("call"), name: name.into(), arguments };
        self.push(ChatResponse { tool_calls: vec![call], model: "fake".into(), ..Default::default() })
    }
    /// Queues any response.
    pub fn push(&self, resp: ChatResponse) -> &Self {
        if let Ok(mut s) = self.script.lock() {
            s.push_back(resp);
        }
        self
    }
    /// Sets the transcript returned by `transcribe`.
    pub fn set_transcript(&self, text: &str) {
        if let Ok(mut t) = self.transcript.lock() {
            *t = Some(text.into());
        }
    }
    /// Every call made so far (role + request), for prompt snapshot tests.
    pub fn calls(&self) -> Vec<(ModelRole, ChatRequest)> {
        self.calls.lock().map(|c| c.clone()).unwrap_or_default()
    }

    fn next(&self, role: ModelRole, req: &ChatRequest) -> ChatResponse {
        if let Ok(mut c) = self.calls.lock() {
            c.push((role, req.clone()));
        }
        let scripted = self.script.lock().ok().and_then(|mut s| s.pop_front());
        scripted.unwrap_or_else(|| {
            let last = req.messages.iter().rev().find(|m| m.role == ChatRole::User);
            ChatResponse {
                content: last.map(|m| m.content.clone()).unwrap_or_default(),
                usage: Usage { prompt_tokens: 10, completion_tokens: 10, cost_usd: 0.0 },
                model: "fake".into(),
                ..Default::default()
            }
        })
    }
}

#[async_trait]
impl ModelGateway for FakeGateway {
    async fn chat(&self, role: ModelRole, req: ChatRequest) -> Result<ChatResponse, ModelError> {
        Ok(self.next(role, &req))
    }

    async fn chat_stream(&self, role: ModelRole, req: ChatRequest) -> Result<ChatStream, ModelError> {
        let resp = self.next(role, &req);
        let mut deltas: Vec<Result<ChatDelta, ModelError>> =
            resp.content.split_inclusive(' ').map(|w| Ok(ChatDelta::Text(w.to_string()))).collect();
        deltas.extend(resp.tool_calls.into_iter().map(|c| Ok(ChatDelta::ToolCall(c))));
        deltas.push(Ok(ChatDelta::Done(resp.usage)));
        Ok(Box::pin(futures::stream::iter(deltas)))
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, ModelError> {
        // Deterministic 8-dim bag-of-bytes embedding: similar texts get similar vectors.
        Ok(texts
            .iter()
            .map(|t| {
                let mut v = [0f32; 8];
                for (i, b) in t.to_lowercase().bytes().enumerate() {
                    v[(b as usize + i) % 8] += 1.0;
                }
                let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-6);
                v.iter().map(|x| x / norm).collect()
            })
            .collect())
    }

    async fn transcribe(&self, file: &FileRef, _bytes: &[u8]) -> Result<Transcript, ModelError> {
        let text = self
            .transcript
            .lock()
            .ok()
            .and_then(|t| t.clone())
            .unwrap_or_else(|| format!("Transcript of {}", file.name));
        Ok(Transcript { text, ..Default::default() })
    }
}
