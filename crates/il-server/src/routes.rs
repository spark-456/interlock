//! Core routes plus every registered `RouteFactory`. B04 adds the rest of PRD §8.5 here.

use axum::extract::State;
use axum::http::{HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use il_core::ApiError;
use il_core::dto::{CatalogResponse, Health};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::assets;
use crate::wiring::AppState;

/// The full router.
pub fn router(state: AppState) -> Router {
    let ctx = state.route_ctx();
    let mut app = Router::new()
        .route("/healthz", get(healthz))
        .route("/api/nodes", get(nodes))
        .merge(crate::api::router())
        .route("/api/{*rest}", get(api_not_found).post(api_not_found).put(api_not_found).delete(api_not_found))
        .with_state(state.clone());
    for factory in il_sdk::registry::route_factories() {
        tracing::info!(group = factory.name, "mounting route group");
        app = app.merge((factory.make)(&ctx));
    }
    app.fallback(assets::serve)
        .layer(cors(&state.config.server.cors_origins))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
}

/// CORS: same-origin by default; listed origins may call with credentials (PRD §6 Security).
fn cors(origins: &[String]) -> CorsLayer {
    let allowed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true)
}

async fn healthz() -> Json<Health> {
    Json(Health { ok: true, version: env!("CARGO_PKG_VERSION").into() })
}

async fn nodes(State(_state): State<AppState>) -> Json<CatalogResponse> {
    Json(CatalogResponse { nodes: il_sdk::registry::catalog() })
}

async fn api_not_found(uri: Uri) -> Response {
    error(StatusCode::NOT_FOUND, "not_found", &format!("No API route for {}.", uri.path()))
}

/// Error response in the PRD §8.5 shape.
pub fn error(status: StatusCode, code: &str, message: &str) -> Response {
    (status, Json(ApiError::new(code, message))).into_response()
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    use super::*;
    use crate::{Config, build_state};

    fn app() -> Router {
        router(build_state(Config::from_value(serde_json::json!({})).unwrap()))
    }

    async fn get_json(path: &str) -> (StatusCode, serde_json::Value) {
        let resp = app().oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null))
    }

    #[tokio::test]
    async fn healthz_ok() {
        let (status, body) = get_json("/healthz").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["ok"], true);
    }

    #[tokio::test]
    async fn catalog_has_every_p0_node() {
        let (status, body) = get_json("/api/nodes").await;
        assert_eq!(status, StatusCode::OK);
        let ids: Vec<&str> = body["nodes"].as_array().unwrap().iter().map(|n| n["typeId"].as_str().unwrap()).collect();
        assert!(ids.len() >= 58, "catalog has {} nodes", ids.len());
        for id in [
            "trigger.schedule",
            "ai.write",
            "ai.generate_video",
            "knowledge.search",
            "media.cut_clips",
            "logic.switch",
            "data.http",
            "data.openapi",
            "human.approval",
            "output.respond",
            "app.gmail.reply",
            "app.instagram.post_reel",
            "app.shopify.get_products",
        ] {
            assert!(ids.contains(&id), "missing {id}");
        }
    }

    #[tokio::test]
    async fn unknown_api_route_uses_error_shape() {
        let (status, body) = get_json("/api/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(body["error"]["code"], "not_found");
    }
}
