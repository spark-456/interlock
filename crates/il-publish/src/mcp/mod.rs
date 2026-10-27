//! Streamable HTTP MCP endpoints backed only by SDK services.

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Request, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use il_core::{Flow, FlowKind};
use il_sdk::registry::{RouteCtx, RouteFactory};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerConfig, Tool,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use rmcp::{ErrorData, ServerHandler};
use serde_json::Value;
use tower::ServiceExt;

use crate::keys::KeyManager;
use crate::openai::slug;

mod flow;
mod platform;

#[derive(Clone)]
enum Scope {
    Platform,
    Flow(Box<Flow>),
}

#[derive(Clone)]
struct McpServer {
    ctx: RouteCtx,
    workspace_id: String,
    scope: Scope,
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_instructions("Use Interlock tools only for flows in this workspace.")
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        let tools = match &self.scope {
            Scope::Platform => platform::tools(),
            Scope::Flow(flow) => {
                let available = self
                    .ctx
                    .flow_tools
                    .for_flow(&self.workspace_id, flow)
                    .await
                    .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
                flow::tools(flow, &available)
            }
        };
        Ok(ListToolsResult::with_all_items(tools))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let arguments = request.arguments.unwrap_or_default();
        let result = match &self.scope {
            Scope::Platform => platform::call(&self.ctx, &self.workspace_id, request.name.as_ref(), &arguments).await,
            Scope::Flow(flow) => match self.ctx.flow_tools.for_flow(&self.workspace_id, flow).await {
                Ok(available) => {
                    flow::call(&self.ctx, &self.workspace_id, flow, request.name.as_ref(), &arguments, &available).await
                }
                Err(error) => Err(error.to_string()),
            },
        };
        let response = match result {
            Ok(Value::String(value)) => CallToolResult::success(vec![ContentBlock::text(value)]),
            Ok(value) => CallToolResult::success(vec![ContentBlock::text(value.to_string())]),
            Err(message) => CallToolResult::error(vec![ContentBlock::text(message)]),
        };
        Ok(response.into())
    }
}

/// Registers platform MCP on `/mcp` and per-flow MCP on `/mcp/:slug`.
pub fn routes(ctx: &RouteCtx) -> Router {
    Router::new()
        .route("/mcp", any(platform_request))
        .route("/mcp/{flow_slug}", any(flow_request))
        .with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "publish.mcp", make: routes } }

async fn platform_request(State(ctx): State<RouteCtx>, headers: HeaderMap, request: Request) -> Response {
    let workspace_id = match workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };
    serve(McpServer { ctx, workspace_id, scope: Scope::Platform }, request).await
}

async fn flow_request(
    State(ctx): State<RouteCtx>,
    Path(flow_slug): Path<String>,
    headers: HeaderMap,
    request: Request,
) -> Response {
    let workspace_id = match workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };
    let flows = match ctx.services.store.list_flows(&workspace_id).await {
        Ok(flows) => flows,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let mut matches = flows.into_iter().filter(|flow| flow.kind == FlowKind::App && slug(flow) == flow_slug);
    let Some(flow) = matches.next() else { return StatusCode::NOT_FOUND.into_response() };
    if matches.next().is_some() {
        return StatusCode::NOT_FOUND.into_response();
    }
    if !ctx.services.store.is_flow_active(&flow.id).await.unwrap_or(false) {
        return StatusCode::NOT_FOUND.into_response();
    }
    serve(McpServer { ctx, workspace_id, scope: Scope::Flow(Box::new(flow)) }, request).await
}

async fn workspace(ctx: &RouteCtx, headers: &HeaderMap) -> Result<String, StatusCode> {
    let raw = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let Some(raw) = raw else { return Err(StatusCode::UNAUTHORIZED) };
    match KeyManager::new(ctx.services.store.clone()).authenticate(raw).await {
        Ok(Some(id)) => Ok(id),
        Ok(None) => Err(StatusCode::UNAUTHORIZED),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn serve(server: McpServer, request: Request) -> Response {
    let config = StreamableHttpServerConfig::default().with_legacy_session_mode(false).with_json_response(true);
    let service = StreamableHttpService::new(move || Ok(server.clone()), LocalSessionManager::default().into(), config);
    match service.oneshot(request).await {
        Ok(response) => response.map(Body::new),
        Err(never) => match never {},
    }
}

fn tool(name: &str, description: &str, schema: Value) -> Tool {
    Tool::new(name.to_owned(), description.to_owned(), schema.as_object().cloned().unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::http::Request;
    use il_core::Flow;
    use il_sdk::registry::RouteCtx;
    use il_sdk::services::{AgentTool, FlowToolProvider, Store, ToolError};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use serde_json::json;

    use super::*;

    struct TestTool(&'static str);

    #[async_trait]
    impl AgentTool for TestTool {
        fn name(&self) -> &str {
            self.0
        }
        fn description(&self) -> &str {
            "Look up an invoice"
        }
        fn schema(&self) -> Value {
            json!({"type":"object","properties":{"id":{"type":"string"}}})
        }
        async fn invoke(&self, args: Value) -> Result<Value, ToolError> {
            Ok(json!({"id":args["id"],"paid":true}))
        }
    }

    struct TestToolProvider;

    #[async_trait]
    impl FlowToolProvider for TestToolProvider {
        async fn for_flow(&self, _workspace_id: &str, _flow: &Flow) -> Result<Vec<Arc<dyn AgentTool>>, ToolError> {
            Ok(vec![Arc::new(TestTool("get_invoice")), Arc::new(TestTool("hidden_tool"))])
        }
    }

    #[tokio::test]
    async fn p01_platform_mcp_requires_a_valid_api_key() {
        let fakes = FakeServices::default();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(TestToolProvider),
            config: json!({}),
        };
        let router = routes(&ctx);
        let body = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{
            "protocolVersion":"2025-11-25","capabilities":{},
            "clientInfo":{"name":"test","version":"1"}
        }})
        .to_string();
        let denied = router
            .clone()
            .oneshot(
                Request::post("/mcp")
                    .header("content-type", "application/json")
                    .body(Body::from(body.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let flow: Flow = serde_json::from_value(json!({"id":"flow_1","workspaceId":"ws_a","version":1,
            "name":"Demo","kind":"app","nodes":[{"id":"n_agent","name":"Agent","type":"ai.agent",
                "settings":{"tools":["get_invoice"]}}],"edges":[]}))
        .unwrap();
        fakes.store.put_flow(&flow).await.unwrap();
        fakes.store.set_flow_active(&flow.id, true).await.unwrap();
        let (_, key) = KeyManager::new(fakes.store.clone()).create("ws_a", "test").await.unwrap();
        let allowed = router
            .clone()
            .oneshot(
                Request::post("/mcp")
                    .header("host", "localhost")
                    .header("content-type", "application/json")
                    .header("accept", "application/json, text/event-stream")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = allowed.status();
        let body = axum::body::to_bytes(allowed.into_body(), 4096).await.unwrap();
        assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        for (method, params, expected) in [
            ("tools/list", json!({}), "list_flows"),
            ("tools/call", json!({"name":"list_flows","arguments":{}}), "flow_1"),
        ] {
            let request = json!({"jsonrpc":"2.0","id":2,"method":method,"params":params});
            let response = router
                .clone()
                .oneshot(
                    Request::post("/mcp")
                        .header("host", "localhost")
                        .header("content-type", "application/json")
                        .header("accept", "application/json, text/event-stream")
                        .header("mcp-protocol-version", "2025-11-25")
                        .header("authorization", format!("Bearer {key}"))
                        .body(Body::from(request.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            let status = response.status();
            let body = axum::body::to_bytes(response.into_body(), 16384).await.unwrap();
            assert_eq!(status, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
            assert!(String::from_utf8_lossy(&body).contains(expected));
        }
        let flow_tools = router
            .clone()
            .oneshot(
                Request::post("/mcp/demo")
                    .header("host", "localhost")
                    .header("content-type", "application/json")
                    .header("accept", "application/json, text/event-stream")
                    .header("mcp-protocol-version", "2025-11-25")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::from(json!({"jsonrpc":"2.0","id":3,"method":"tools/list","params":{}}).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let flow_body = axum::body::to_bytes(flow_tools.into_body(), 16384).await.unwrap();
        assert!(String::from_utf8_lossy(&flow_body).contains("ask_demo"));
        assert!(String::from_utf8_lossy(&flow_body).contains("get_invoice"));
        assert!(!String::from_utf8_lossy(&flow_body).contains("hidden_tool"));
        let invoice = router
            .clone()
            .oneshot(
                Request::post("/mcp/demo")
                    .header("host", "localhost")
                    .header("content-type", "application/json")
                    .header("accept", "application/json, text/event-stream")
                    .header("mcp-protocol-version", "2025-11-25")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::from(
                        json!({"jsonrpc":"2.0","id":5,"method":"tools/call",
                    "params":{"name":"get_invoice","arguments":{"id":"inv_1"}}})
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let invoice_body = axum::body::to_bytes(invoice.into_body(), 16384).await.unwrap();
        assert!(String::from_utf8_lossy(&invoice_body).contains("inv_1"));
        let asked = router
            .clone()
            .oneshot(
                Request::post("/mcp/demo")
                    .header("host", "localhost")
                    .header("content-type", "application/json")
                    .header("accept", "application/json, text/event-stream")
                    .header("mcp-protocol-version", "2025-11-25")
                    .header("authorization", format!("Bearer {key}"))
                    .body(Body::from(
                        json!({"jsonrpc":"2.0","id":4,"method":"tools/call",
                "params":{"name":"ask_demo","arguments":{"message":"hello"}}})
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        let answer = axum::body::to_bytes(asked.into_body(), 16384).await.unwrap();
        assert!(String::from_utf8_lossy(&answer).contains("hello"));
        let (_, other_key) = KeyManager::new(fakes.store).create("ws_b", "other").await.unwrap();
        let denied = router
            .oneshot(
                Request::post("/mcp/demo")
                    .header("authorization", format!("Bearer {other_key}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(denied.status(), StatusCode::NOT_FOUND);
    }
}
