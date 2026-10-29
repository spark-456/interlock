//! Server-sent events from a streaming `/chat/completions` call, as `ChatDelta`s.

use std::collections::{BTreeMap, VecDeque};

use bytes::Bytes;
use futures::stream::{BoxStream, StreamExt};
use il_sdk::services::models::{ChatDelta, ChatStream, ModelError, ToolCall, Usage};
use serde_json::Value;

use super::wire;

struct State {
    body: BoxStream<'static, reqwest::Result<Bytes>>,
    buf: String,
    out: VecDeque<Result<ChatDelta, ModelError>>,
    /// Tool calls arrive in pieces, keyed by index: (id, name, arguments so far).
    tools: BTreeMap<u64, (String, String, String)>,
    usage: Usage,
    done: bool,
}

/// Parses the response body; tool calls are emitted whole just before `Done`.
pub fn parse(body: BoxStream<'static, reqwest::Result<Bytes>>) -> ChatStream {
    let state = State {
        body,
        buf: String::new(),
        out: VecDeque::new(),
        tools: BTreeMap::new(),
        usage: Usage::default(),
        done: false,
    };
    Box::pin(futures::stream::unfold(state, |mut s| async move {
        loop {
            if let Some(next) = s.out.pop_front() {
                return Some((next, s));
            }
            if s.done {
                return None;
            }
            match s.body.next().await {
                Some(Ok(chunk)) => {
                    s.buf.push_str(&String::from_utf8_lossy(&chunk));
                    while let Some(nl) = s.buf.find('\n') {
                        let line: String = s.buf.drain(..=nl).collect();
                        s.line(line.trim());
                    }
                }
                Some(Err(e)) => {
                    s.out.push_back(Err(ModelError::Unavailable(e.to_string())));
                    s.done = true;
                }
                None => s.finish(),
            }
        }
    }))
}

impl State {
    fn line(&mut self, line: &str) {
        let Some(data) = line.strip_prefix("data:").map(str::trim) else { return };
        if data == "[DONE]" {
            return self.finish();
        }
        let Ok(v) = serde_json::from_str::<Value>(data) else { return };
        if v.get("usage").is_some_and(|u| !u.is_null()) {
            self.usage = wire::usage(v.get("usage"));
        }
        let Some(delta) = v.pointer("/choices/0/delta") else { return };
        if let Some(text) = delta.get("content").and_then(Value::as_str).filter(|t| !t.is_empty()) {
            self.out.push_back(Ok(ChatDelta::Text(text.to_string())));
        }
        for call in delta.get("tool_calls").and_then(Value::as_array).into_iter().flatten() {
            let entry = self.tools.entry(call.get("index").and_then(Value::as_u64).unwrap_or(0)).or_default();
            let piece = |k: &str| call.get(k).and_then(Value::as_str);
            if let Some(id) = piece("id") {
                entry.0 = id.to_string();
            }
            if let Some(f) = call.get("function") {
                entry.1.push_str(f.get("name").and_then(Value::as_str).unwrap_or_default());
                entry.2.push_str(f.get("arguments").and_then(Value::as_str).unwrap_or_default());
            }
        }
    }

    fn finish(&mut self) {
        if self.done {
            return;
        }
        for (_, (id, name, args)) in std::mem::take(&mut self.tools) {
            self.out.push_back(Ok(ChatDelta::ToolCall(ToolCall { id, name, arguments: wire::parse_arguments(&args) })));
        }
        self.out.push_back(Ok(ChatDelta::Done(self.usage)));
        self.done = true;
    }
}
