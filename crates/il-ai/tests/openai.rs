//! The OpenAI-compatible provider against a local mock server.

use futures::StreamExt;
use il_ai::providers::openai::OpenAiCompat;
use il_sdk::SafeHttp;
use il_sdk::services::models::{ChatDelta, ChatMessage, ChatRequest, ModelError, ModelProvider, ToolSpec};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn provider() -> (MockServer, OpenAiCompat) {
    let server = MockServer::start().await;
    let tf = OpenAiCompat::new(
        "tokenfactory",
        "NEBIUS_API_KEY",
        SafeHttp::permissive_for_tests(),
        &format!("{}/v1/", server.uri()),
        "key".into(),
    );
    (server, tf)
}

fn ask() -> ChatRequest {
    ChatRequest { messages: vec![ChatMessage::user("invoice 7?")], ..Default::default() }
}

#[tokio::test]
async fn chat_sends_tools_and_schema_and_reads_tool_calls() {
    let (server, tf) = provider().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer key"))
        .and(body_partial_json(json!({
            "model": "m",
            "tools": [{"type": "function", "function": {"name": "get_invoice"}}],
            "response_format": {"type": "json_schema"},
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "m",
            "choices": [{"message": {"content": null, "tool_calls": [
                {"id": "c1", "type": "function", "function": {"name": "get_invoice", "arguments": "{\"id\":7}"}}
            ]}}],
            "usage": {"prompt_tokens": 12, "completion_tokens": 3},
        })))
        .mount(&server)
        .await;
    let tool = ToolSpec { name: "get_invoice".into(), description: "Look up".into(), parameters: json!({}) };
    let req = ChatRequest { tools: vec![tool], json_schema: Some(json!({"type": "object"})), ..ask() };
    let resp = tf.chat("m", req).await.unwrap();
    assert_eq!(resp.tool_calls[0].id, "c1");
    assert_eq!(resp.tool_calls[0].arguments, json!({"id": 7}));
    assert_eq!((resp.usage.prompt_tokens, resp.usage.completion_tokens), (12, 3));
}

#[tokio::test]
async fn stream_yields_text_tool_calls_and_usage() {
    let (server, tf) = provider().await;
    let sse = [
        r#"{"choices":[{"delta":{"content":"Hel"}}]}"#,
        r#"{"choices":[{"delta":{"content":"lo"}}]}"#,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"id":"c1","function":{"name":"lookup","arguments":"{\"q\":"}}]}}]}"#,
        r#"{"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"x\"}"}}]}}]}"#,
        r#"{"choices":[],"usage":{"prompt_tokens":5,"completion_tokens":2}}"#,
        "[DONE]",
    ]
    .iter()
    .map(|d| format!("data: {d}\n\n"))
    .collect::<String>();
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(body_partial_json(json!({"stream": true, "stream_options": {"include_usage": true}})))
        .respond_with(ResponseTemplate::new(200).set_body_raw(sse, "text/event-stream"))
        .mount(&server)
        .await;
    let chunks: Vec<ChatDelta> = tf.chat_stream("m", ask()).await.unwrap().map(Result::unwrap).collect().await;
    assert_eq!(chunks[0], ChatDelta::Text("Hel".into()));
    assert_eq!(chunks[1], ChatDelta::Text("lo".into()));
    assert!(matches!(&chunks[2], ChatDelta::ToolCall(c) if c.name == "lookup" && c.arguments == json!({"q": "x"})));
    assert!(matches!(&chunks[3], ChatDelta::Done(u) if u.prompt_tokens == 5 && u.completion_tokens == 2));
}

#[tokio::test]
async fn maps_http_errors() {
    let (server, tf) = provider().await;
    for (status, body) in [
        (429, "slow down"),
        (400, "tools are not supported for this model"),
        (401, "bad key"),
        (402, "no credit"),
        (404, "no model"),
        (503, "down"),
    ] {
        server.reset().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(status).set_body_string(body))
            .mount(&server)
            .await;
        let err = tf.chat("m", ask()).await.unwrap_err();
        let ok = match status {
            429 => matches!(err, ModelError::RateLimited),
            400 => matches!(err, ModelError::Unsupported(_)),
            401 | 404 => matches!(err, ModelError::Config(_)),
            _ => matches!(err, ModelError::Unavailable(_)),
        };
        assert!(ok, "{status} → {err:?}");
    }
}

#[tokio::test]
async fn embeddings_models_and_transcription() {
    let (server, tf) = provider().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [
            {"index": 1, "embedding": [0.5, 0.25]}, {"index": 0, "embedding": [1.0, 0.0]}
        ]})))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data": [{"id": "a"}, {"id": "b"}]})))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "text": "hello there", "language": "en", "segments": [{"start": 0.0, "end": 1.5, "text": " hello there"}]
        })))
        .mount(&server)
        .await;

    let v = tf.embed("e", &["first".into(), "second".into()]).await.unwrap();
    assert_eq!(v, vec![vec![1.0, 0.0], vec![0.5, 0.25]]);
    assert_eq!(tf.list_models().await.unwrap(), vec!["a", "b"]);
    let file = il_core::FileRef {
        id: "f".into(),
        mime: "audio/mpeg".into(),
        bytes: 3,
        name: "a.mp3".into(),
        preview_url: None,
    };
    let t = tf.transcribe("whisper", &file, b"abc").await.unwrap();
    assert_eq!(t.text, "hello there");
    assert_eq!(t.segments[0].text, "hello there");
    assert_eq!(t.language.as_deref(), Some("en"));
}
