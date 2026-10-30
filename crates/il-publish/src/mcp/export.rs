//! `GET /mcp/flows/:id/export`: a flow's self-hosting bundle as a zip, for the API key's workspace,
//! so a coding agent can pull it into a repository (`export_flow` hands it the command).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use il_sdk::registry::RouteCtx;

pub(super) async fn download(State(ctx): State<RouteCtx>, Path(flow_id): Path<String>, headers: HeaderMap) -> Response {
    let workspace_id = match super::workspace(&ctx, &headers).await {
        Ok(id) => id,
        Err(status) => return status.into_response(),
    };
    match ctx.services.store.get_flow(&flow_id).await {
        Ok(Some(flow)) if flow.workspace_id == workspace_id => {
            crate::export::response(&ctx.services, &ctx.config, &flow).await
        }
        Ok(_) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::Request;
    use il_core::Flow;
    use il_sdk::services::{NoActivation, NoFlowTools, Store};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use serde_json::json;
    use tower::ServiceExt;

    use super::*;
    use crate::keys::KeyManager;

    #[tokio::test]
    async fn q36_agents_download_bundles_with_the_workspace_key_and_get_the_command() {
        let fakes = FakeServices::default();
        let ctx = RouteCtx {
            services: fakes.services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(NoFlowTools),
            activation: Arc::new(NoActivation),
            config: json!({"server": {"public_url": "https://interlock.example"}}),
        };
        for (id, ws) in [("flw_a", "ws_a"), ("flw_b", "ws_b")] {
            let flow: Flow = serde_json::from_value(json!({"id": id, "workspaceId": ws, "version": 1, "name": "Help",
                "kind": "app", "nodes": [{"id": "n_chat", "type": "trigger.chat", "name": "Chat", "settings": {}}],
                "edges": []}))
            .unwrap();
            fakes.store.put_flow(&flow).await.unwrap();
        }
        let (_, key) = KeyManager::new(fakes.store.clone()).create("ws_a", "agent").await.unwrap();
        let get = |flow: &str, auth: bool| {
            let mut req = Request::get(format!("/mcp/flows/{flow}/export"));
            if auth {
                req = req.header("authorization", format!("Bearer {key}"));
            }
            super::super::routes(&ctx).oneshot(req.body(Body::empty()).unwrap())
        };
        assert_eq!(get("flw_a", false).await.unwrap().status(), StatusCode::UNAUTHORIZED);
        assert_eq!(get("flw_b", true).await.unwrap().status(), StatusCode::NOT_FOUND, "another workspace's flow");
        let ok = get("flw_a", true).await.unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        assert_eq!(ok.headers()["content-type"], "application/zip");

        let args = |id: &str| json!({"flow_id": id}).as_object().cloned().unwrap();
        let tool = crate::mcp::platform::call(&ctx, "ws_a", "export_flow", &args("flw_a")).await.unwrap();
        let command = tool["command"].as_str().unwrap();
        assert!(command.contains("https://interlock.example/mcp/flows/flw_a/export") && command.contains("unzip"));
        assert!(tool["files"].as_array().unwrap().iter().any(|f| f == "INTERLOCK.md"));
        assert!(tool["settings"].as_array().unwrap().iter().any(|s| s["name"] == "INTERLOCK_API_KEY"));
        let foreign = crate::mcp::platform::call(&ctx, "ws_a", "export_flow", &args("flw_b")).await;
        assert_eq!(foreign.unwrap_err(), "flow not found");
    }
}
