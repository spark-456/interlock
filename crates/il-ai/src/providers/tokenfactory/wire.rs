//! OpenAI-compatible request bodies and response parsing.

use il_sdk::services::models::{ChatRequest, ChatResponse, ChatRole, ModelError, ToolCall, Usage};
use serde_json::{Value, json};

/// The `/chat/completions` body for `req`.
pub fn chat_body(model: &str, req: &ChatRequest, stream: bool) -> Value {
    let messages: Vec<Value> = req
        .messages
        .iter()
        .map(|m| {
            let content = match &m.image_url {
                Some(url) => json!([
                    {"type":"text","text":m.content},
                    {"type":"image_url","image_url":{"url":url}}
                ]),
                None => json!(m.content),
            };
            let mut msg = json!({"role": role(m.role), "content": content});
            if !m.tool_calls.is_empty() {
                msg["tool_calls"] = m
                    .tool_calls
                    .iter()
                    .map(|c| json!({"id": c.id, "type": "function", "function": {"name": c.name, "arguments": c.arguments.to_string()}}))
                    .collect();
            }
            if let Some(id) = &m.tool_call_id {
                msg["tool_call_id"] = json!(id);
            }
            msg
        })
        .collect();
    let mut body = json!({"model": model, "messages": messages});
    if !req.tools.is_empty() {
        body["tools"] = req
            .tools
            .iter()
            .map(|t| json!({"type": "function", "function": {"name": t.name, "description": t.description, "parameters": t.parameters}}))
            .collect();
    }
    if let Some(schema) = &req.json_schema {
        body["response_format"] =
            json!({"type": "json_schema", "json_schema": {"name": "output", "schema": schema, "strict": true}});
    }
    if let Some(t) = req.temperature {
        body["temperature"] = json!(t);
    }
    if let Some(n) = req.max_tokens {
        body["max_tokens"] = json!(n);
    }
    if stream {
        body["stream"] = json!(true);
        body["stream_options"] = json!({"include_usage": true});
    }
    body
}

fn role(r: ChatRole) -> &'static str {
    match r {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
        ChatRole::Tool => "tool",
    }
}

/// A `/chat/completions` response.
pub fn chat_response(v: &Value) -> Result<ChatResponse, ModelError> {
    let msg =
        v.pointer("/choices/0/message").ok_or_else(|| ModelError::InvalidOutput("no choices in the reply".into()))?;
    let tool_calls = msg
        .get("tool_calls")
        .and_then(Value::as_array)
        .map(|calls| calls.iter().filter_map(tool_call).collect())
        .unwrap_or_default();
    Ok(ChatResponse {
        content: msg.get("content").and_then(Value::as_str).unwrap_or_default().to_string(),
        tool_calls,
        usage: usage(v.get("usage")),
        model: v.get("model").and_then(Value::as_str).unwrap_or_default().to_string(),
    })
}

fn tool_call(c: &Value) -> Option<ToolCall> {
    let f = c.get("function")?;
    Some(ToolCall {
        id: c.get("id").and_then(Value::as_str).unwrap_or_default().to_string(),
        name: f.get("name")?.as_str()?.to_string(),
        arguments: parse_arguments(f.get("arguments").and_then(Value::as_str).unwrap_or("{}")),
    })
}

/// Tool arguments arrive as a JSON string; keep the raw text if the model wrote invalid JSON.
pub fn parse_arguments(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
}

/// Token counts from a `usage` object (cost is added by the gateway).
pub fn usage(u: Option<&Value>) -> Usage {
    let n = |k: &str| u.and_then(|u| u.get(k)).and_then(Value::as_u64).unwrap_or(0) as u32;
    Usage { prompt_tokens: n("prompt_tokens"), completion_tokens: n("completion_tokens"), cost_usd: 0.0 }
}

/// Maps an HTTP error to the gateway's errors; a 400 about tools or schemas means "fall back".
pub fn error(status: u16, body: &str) -> ModelError {
    let text: String = body.chars().take(300).collect();
    let lower = text.to_lowercase();
    match status {
        429 => ModelError::RateLimited,
        401 | 403 => ModelError::Config("Token Factory rejected the API key (NEBIUS_API_KEY)".into()),
        400 | 422 if ["tool", "response_format", "json_schema", "not supported"].iter().any(|k| lower.contains(k)) => {
            ModelError::Unsupported(text)
        }
        s if s >= 500 => ModelError::Unavailable(format!("HTTP {s}: {text}")),
        s => ModelError::Other(format!("HTTP {s}: {text}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use il_sdk::services::models::ChatMessage;

    #[test]
    fn b07_image_message_uses_multimodal_content() {
        let mut message = ChatMessage::user("Describe this");
        message.image_url = Some("data:image/png;base64,AQID".into());
        let body = chat_body("vision", &ChatRequest { messages: vec![message], ..Default::default() }, false);
        assert_eq!(body["messages"][0]["content"][0]["text"], "Describe this");
        assert_eq!(body["messages"][0]["content"][1]["image_url"]["url"], "data:image/png;base64,AQID");
    }
}
