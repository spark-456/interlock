//! Serves the chat widget bundle at `/w/:app.js`. The bundle reads the app from its own URL
//! and posts chat turns to `/w/:app/chat`.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use il_core::{ApiError, FlowKind};
use il_sdk::registry::{RouteCtx, RouteFactory};

use crate::openai::{self, ChatRequest, slug};

/// Built from `web/widget/` by `pnpm -C web build:widget`.
const BUNDLE: &str = include_str!("../../../../web/widget/dist/widget.js");

/// Registers the widget script and public chat routes.
pub fn routes(ctx: &RouteCtx) -> Router {
    Router::new().route("/w/{file}", get(script)).route("/w/{app}/chat", post(chat)).with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "publish.widget", make: routes } }

async fn script(Path(file): Path<String>) -> Response {
    if !file.ends_with(".js") {
        return StatusCode::NOT_FOUND.into_response();
    }
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=300")], BUNDLE)
        .into_response()
}

async fn chat(State(ctx): State<RouteCtx>, Path(app): Path<String>, Json(request): Json<ChatRequest>) -> Response {
    if request.model != app {
        return (StatusCode::BAD_REQUEST, Json(ApiError::new("bad_request", "Model must match the widget app.")))
            .into_response();
    }
    let flows = match ctx.services.store.active_flows().await {
        Ok(flows) => flows,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let mut matches = flows.into_iter().filter(|flow| flow.kind == FlowKind::App && slug(flow) == app);
    let Some(flow) = matches.next() else { return StatusCode::NOT_FOUND.into_response() };
    if matches.next().is_some() {
        return StatusCode::NOT_FOUND.into_response();
    }
    openai::respond(&ctx, &flow, request, "widget").await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use il_core::Flow;
    use il_sdk::services::Store;
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use serde_json::json;
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_the_bundle_for_any_app_script() {
        let ctx = RouteCtx {
            services: FakeServices::default().services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            activation: std::sync::Arc::new(il_sdk::services::NoActivation),
            config: serde_json::json!({}),
        };
        let app = routes(&ctx);
        let ok = app.clone().oneshot(Request::get("/w/support-agent.js").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        assert!(ok.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/javascript"));
        let missing = app.oneshot(Request::get("/w/support-agent").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn p01_widget_chat_only_runs_a_published_app() {
        let fakes = FakeServices::default();
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws_a","version":1,
            "name":"Support Agent","kind":"app","nodes":[],"edges":[]}))
        .unwrap();
        fakes.store.put_flow(&flow).await.unwrap();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            activation: std::sync::Arc::new(il_sdk::services::NoActivation),
            config: json!({}),
        };
        let app = routes(&ctx);
        let body = json!({"model":"support-agent","stream":true,
            "messages":[{"role":"user","content":"hello"}]})
        .to_string();
        let send = |body: String| {
            Request::post("/w/support-agent/chat")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap()
        };
        let inactive = app.clone().oneshot(send(body.clone())).await.unwrap();
        assert_eq!(inactive.status(), StatusCode::NOT_FOUND);
        fakes.store.set_flow_active(&flow.id, true).await.unwrap();
        let mismatch = app.clone().oneshot(send(body.replace("support-agent", "another-app"))).await.unwrap();
        assert_eq!(mismatch.status(), StatusCode::BAD_REQUEST);
        let active = app.oneshot(send(body)).await.unwrap();
        assert_eq!(active.status(), StatusCode::OK);
        assert!(active.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/event-stream"));
        let text = String::from_utf8_lossy(&to_bytes(active.into_body(), 100_000).await.unwrap()).into_owned();
        assert!(text.contains("hello"));
        assert!(text.contains("[DONE]"));
    }
}
