//! JSON-in-prompt protocol for models without native tool calling or JSON-schema output.

use il_sdk::services::models::{ChatMessage, ChatRequest, ChatResponse, ToolCall};
use serde_json::{Value, json};

/// The request rewritten so tools and the schema are described in a system message instead.
pub fn prompt(req: &ChatRequest) -> ChatRequest {
    let mut rules = Vec::new();
    if !req.tools.is_empty() {
        let tools: Vec<Value> = req
            .tools
            .iter()
            .map(|t| json!({"name": t.name, "description": t.description, "parameters": t.parameters}))
            .collect();
        rules.push(format!(
            "You can call these tools:\n{}\nTo call tools, reply with only {{\"tool_calls\": [{{\"name\": \"…\", \"arguments\": {{…}}}}]}}.",
            Value::Array(tools)
        ));
    }
    if let Some(schema) = &req.json_schema {
        rules.push(format!(
            "Otherwise reply with only a JSON value matching this JSON Schema, with no prose:\n{schema}"
        ));
    }
    let mut out = ChatRequest { tools: vec![], json_schema: None, ..req.clone() };
    if !rules.is_empty() {
        out.messages.insert(0, ChatMessage::system(rules.join("\n\n")));
    }
    out
}

/// Reads tool calls written by the model under the protocol above into `resp.tool_calls`.
pub fn parse_tool_calls(resp: &mut ChatResponse) {
    let Some(calls) = extract_json(&resp.content).and_then(|v| v.get("tool_calls").and_then(Value::as_array).cloned())
    else {
        return;
    };
    resp.tool_calls = calls
        .iter()
        .enumerate()
        .filter_map(|(i, c)| {
            let name = c.get("name")?.as_str()?.to_string();
            Some(ToolCall {
                id: format!("call_{i}"),
                name,
                arguments: c.get("arguments").cloned().unwrap_or(json!({})),
            })
        })
        .collect();
    if !resp.tool_calls.is_empty() {
        resp.content.clear();
    }
}

/// The JSON value in a model reply, tolerating code fences and text around it.
pub fn extract_json(text: &str) -> Option<Value> {
    let t = text.trim();
    let t = t.strip_prefix("```json").or_else(|| t.strip_prefix("```")).map_or(t, |s| s.trim_end_matches("```")).trim();
    if let Ok(v) = serde_json::from_str(t) {
        return Some(v);
    }
    let start = t.find(['{', '['])?;
    let end = t.rfind(['}', ']'])?;
    serde_json::from_str(t.get(start..=end)?).ok()
}

#[cfg(test)]
mod tests {
    use il_sdk::services::models::ToolSpec;

    use super::*;

    #[test]
    fn extracts_json_from_fences_and_prose() {
        assert_eq!(extract_json("```json\n{\"a\": 1}\n```"), Some(json!({"a": 1})));
        assert_eq!(extract_json("Sure! {\"a\": [1]} hope that helps"), Some(json!({"a": [1]})));
        assert_eq!(extract_json("no json here"), None);
    }

    #[test]
    fn describes_tools_and_parses_calls() {
        let tool = ToolSpec { name: "get_invoice".into(), description: "Look up".into(), parameters: json!({}) };
        let req = ChatRequest { messages: vec![ChatMessage::user("hi")], tools: vec![tool], ..Default::default() };
        let p = prompt(&req);
        assert!(p.tools.is_empty() && p.messages[0].content.contains("get_invoice"));
        let mut resp = ChatResponse {
            content: r#"{"tool_calls": [{"name": "get_invoice", "arguments": {"id": 7}}]}"#.into(),
            ..Default::default()
        };
        parse_tool_calls(&mut resp);
        assert_eq!(resp.tool_calls[0].name, "get_invoice");
        assert_eq!(resp.tool_calls[0].arguments, json!({"id": 7}));
        assert!(resp.content.is_empty());
    }
}
