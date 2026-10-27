//! OpenAI-compatible routes backed by published AI app flows.

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, Sse},
    },
    routing::{get, post},
};
use futures::stream;
use il_core::{Flow, FlowKind, Item};
use il_sdk::{
    registry::{RouteCtx, RouteFactory},
    services::{StartRun, runner::InvokeOutcome},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::keys::KeyManager;

/// Registers the public OpenAI-compatible endpoints.
pub fn routes(ctx: &RouteCtx) -> Router {
    Router::new().route("/v1/models", get(models)).route("/v1/chat/completions", post(chat)).with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "publish.openai", make: routes } }

#[derive(Deserialize)]
pub(crate) struct ChatRequest {
    pub(crate) model: String,
    pub(crate) messages: Vec<Message>,
    #[serde(default)]
    pub(crate) stream: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Message {
    role: String,
    content: Value,
}

async fn workspace(ctx: &RouteCtx, headers: &HeaderMap) -> Result<String, (StatusCode, &'static str)> {
    let raw = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "missing Bearer API key"))?;
    KeyManager::new(ctx.services.store.clone())
        .authenticate(raw)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "API key lookup failed"))?
        .ok_or((StatusCode::UNAUTHORIZED, "invalid API key"))
}

async fn published(ctx: &RouteCtx, workspace_id: &str) -> Result<Vec<Flow>, (StatusCode, &'static str)> {
    let flows = ctx
        .services
        .store
        .list_flows(workspace_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "flow lookup failed"))?;
    let mut output = Vec::new();
    for flow in flows {
        if flow.kind == FlowKind::App && ctx.services.store.is_flow_active(&flow.id).await.unwrap_or(false) {
            output.push(flow);
        }
    }
    Ok(output)
}

pub(crate) fn slug(flow: &Flow) -> String {
    let value = flow
        .name
        .to_ascii_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    value.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-")
}

async fn models(State(ctx): State<RouteCtx>, headers: HeaderMap) -> Response {
    let workspace_id = match workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err((status, message)) => return error(status, message),
    };
    let flows = match published(&ctx, &workspace_id).await {
        Ok(flows) => flows,
        Err((status, message)) => return error(status, message),
    };
    Json(json!({"object":"list","data":flows.iter().map(|flow| json!({"id":slug(flow),"object":"model","owned_by":"interlock"})).collect::<Vec<_>>() })).into_response()
}

async fn chat(State(ctx): State<RouteCtx>, headers: HeaderMap, Json(request): Json<ChatRequest>) -> Response {
    let workspace_id = match workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err((status, message)) => return error(status, message),
    };
    let flows = match published(&ctx, &workspace_id).await {
        Ok(flows) => flows,
        Err((status, message)) => return error(status, message),
    };
    let matches: Vec<_> = flows.iter().filter(|flow| slug(flow) == request.model).collect();
    if matches.len() != 1 {
        return error(StatusCode::NOT_FOUND, "published model not found or ambiguous");
    }
    respond(&ctx, matches[0], request, "api").await
}

pub(crate) async fn respond(ctx: &RouteCtx, flow: &Flow, request: ChatRequest, trigger: &str) -> Response {
    let message = request
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .and_then(|message| message.content.as_str())
        .ok_or_else(|| error(StatusCode::BAD_REQUEST, "a user text message is required"));
    let message = match message {
        Ok(message) => message,
        Err(error) => return error,
    };
    let input = Item::json(json!({"message":message,"messages":request.messages}));
    let outcome = ctx
        .runner
        .invoke(StartRun {
            flow_id: flow.id.clone(),
            version: None,
            trigger: trigger.into(),
            items: vec![input],
            trigger_node: None,
        })
        .await;
    let outcome = match outcome {
        Ok(outcome) => outcome,
        Err(_) => return error(StatusCode::BAD_GATEWAY, "flow execution failed"),
    };
    completion(&request.model, request.stream, outcome)
}

fn completion(model: &str, stream: bool, outcome: InvokeOutcome) -> Response {
    let answer = outcome
        .response
        .first()
        .and_then(|item| item.json.get("answer").or_else(|| item.json.get("message")))
        .map(|value| value.as_str().map(str::to_owned).unwrap_or_else(|| value.to_string()))
        .unwrap_or_default();
    let sources = outcome
        .response
        .first()
        .and_then(|item| item.json.get("sources"))
        .filter(|value| value.is_array())
        .cloned()
        .unwrap_or_else(|| json!([]));
    let id = format!("chatcmpl-{}", outcome.run.id);
    let created = chrono::Utc::now().timestamp();
    if stream {
        let first = json!({"id":id,"object":"chat.completion.chunk","created":created,"model":model,
            "choices":[{"index":0,"delta":{"role":"assistant","content":answer},"finish_reason":null}]})
        .to_string();
        let last = json!({"id":id,"object":"chat.completion.chunk","created":created,"model":model,
            "choices":[{"index":0,"delta":{},"finish_reason":"stop"}],"sources":sources})
        .to_string();
        Sse::new(stream::iter(vec![
            Ok::<_, std::convert::Infallible>(Event::default().data(first)),
            Ok(Event::default().data(last)),
            Ok(Event::default().data("[DONE]")),
        ]))
        .into_response()
    } else {
        Json(json!({"id":id,"object":"chat.completion","created":created,"model":model,
            "choices":[{"index":0,"message":{"role":"assistant","content":answer},"finish_reason":"stop"}],"sources":sources}))
        .into_response()
    }
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error":{"message":message,"type":"invalid_request_error"}}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use il_sdk::{
        registry::RouteCtx,
        services::{FlowRunner, Store},
        testkit::{FakeArchitect, FakeRunner, FakeServices},
    };
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn p01_openai_models_and_chat_are_key_scoped() {
        let fakes = FakeServices::default();
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws_a","version":1,
            "name":"Support Agent","kind":"app","nodes":[],"edges":[]}))
        .unwrap();
        fakes.store.put_flow(&flow).await.unwrap();
        fakes.store.set_flow_active(&flow.id, true).await.unwrap();
        let (_, key) = KeyManager::new(fakes.store.clone()).create("ws_a", "Test").await.unwrap();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            config: json!({}),
        };
        let app = routes(&ctx);
        let response = app
            .clone()
            .oneshot(
                Request::get("/v1/models")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let data: Value = serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(data["data"][0]["id"], "support-agent");
        let response = app
            .clone()
            .oneshot(
                Request::post("/v1/chat/completions")
                    .header("authorization", format!("Bearer {key}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"model":"support-agent","messages":[{"role":"user","content":"Hello"}]}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let data: Value = serde_json::from_slice(&to_bytes(response.into_body(), 100_000).await.unwrap()).unwrap();
        assert_eq!(data["choices"][0]["message"]["content"], "Hello");
        let streamed = app
            .clone()
            .oneshot(
                Request::post("/v1/chat/completions")
                    .header("authorization", format!("Bearer {key}"))
                    .header("content-type", "application/json")
                    .body(Body::from(
                        json!({"model":"support-agent","stream":true,
                "messages":[{"role":"user","content":"Hello"}]})
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(streamed.status(), StatusCode::OK);
        assert!(streamed.headers()["content-type"].to_str().unwrap().starts_with("text/event-stream"));
        let body = to_bytes(streamed.into_body(), 100_000).await.unwrap();
        assert!(String::from_utf8_lossy(&body).contains("[DONE]"));
        let denied = app.oneshot(Request::get("/v1/models").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn p01_stream_finishes_with_sources() {
        let outcome = FakeRunner::default()
            .invoke(StartRun {
                flow_id: "flow_1".into(),
                version: None,
                trigger: "widget".into(),
                trigger_node: None,
                items: vec![Item::json(json!({"answer":"Refunds in 30 days","sources":[
                {"title":"Policy","url":"https://example.com/policy"}]}))],
            })
            .await
            .unwrap();
        let response = completion("support-agent", true, outcome);
        let body = to_bytes(response.into_body(), 100_000).await.unwrap();
        let text = String::from_utf8_lossy(&body);
        assert!(text.contains("Refunds in 30 days"));
        assert!(text.contains("\"sources\":[{\"title\":\"Policy\",\"url\":\"https://example.com/policy\"}]"));
        assert!(text.contains("[DONE]"));
    }
}
