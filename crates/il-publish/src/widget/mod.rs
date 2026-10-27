//! Serves the chat widget bundle at `/w/:app.js`. The bundle reads the app from its own URL
//! and posts chat turns to `/w/:app/chat`.

use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use il_sdk::registry::{RouteCtx, RouteFactory};

/// Built from `web/widget/` by `pnpm -C web build:widget`.
const BUNDLE: &str = include_str!("../../../../web/widget/dist/widget.js");

/// Registers the widget script route.
pub fn routes(_ctx: &RouteCtx) -> Router {
    Router::new().route("/w/{file}", get(script))
}

inventory::submit! { RouteFactory { name: "publish.widget", make: routes } }

async fn script(Path(file): Path<String>) -> Response {
    if !file.ends_with(".js") {
        return StatusCode::NOT_FOUND.into_response();
    }
    ([(header::CONTENT_TYPE, "text/javascript; charset=utf-8"), (header::CACHE_CONTROL, "public, max-age=300")], BUNDLE)
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use il_sdk::testkit::{FakeArchitect, FakeRunner, FakeServices};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn serves_the_bundle_for_any_app_script() {
        let ctx = RouteCtx {
            services: FakeServices::default().services(),
            runner: Arc::new(FakeRunner::default()),
            architect: Arc::new(FakeArchitect),
            flow_tools: Arc::new(il_sdk::services::NoFlowTools),
            config: serde_json::json!({}),
        };
        let app = routes(&ctx);
        let ok = app.clone().oneshot(Request::get("/w/support-agent.js").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        assert!(ok.headers()[header::CONTENT_TYPE].to_str().unwrap().starts_with("text/javascript"));
        let missing = app.oneshot(Request::get("/w/support-agent").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    }
}
