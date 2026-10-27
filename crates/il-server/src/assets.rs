//! The embedded web build (`web/build`, from `pnpm -C web build`) with SPA fallback.
//! In debug builds rust-embed reads from disk, so a rebuilt web app shows up without recompiling.

use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../../web/build"]
#[allow_missing = true]
struct WebBuild;

/// Backend prefixes: an unknown path under one is a 404, never the web app.
const BACKEND: [&str; 6] = ["api/", "v1/", "mcp", "hooks/", "w/", "healthz"];

/// Serves a static file, falling back to `index.html` for client-side routes.
pub async fn serve(uri: Uri) -> Response {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };
    if let Some(file) = WebBuild::get(path) {
        return file_response(path, file);
    }
    if BACKEND.iter().any(|p| path.starts_with(p)) {
        return StatusCode::NOT_FOUND.into_response();
    }
    match WebBuild::get("index.html") {
        Some(index) => file_response("index.html", index),
        None => (StatusCode::NOT_FOUND, "The web app is not built. Run `pnpm -C web build`, or use `cargo xtask dev`.")
            .into_response(),
    }
}

fn file_response(path: &str, file: rust_embed::EmbeddedFile) -> Response {
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache = if path.starts_with("_app/immutable/") { "public, max-age=31536000, immutable" } else { "no-cache" };
    ([(header::CONTENT_TYPE, mime.as_ref().to_string()), (header::CACHE_CONTROL, cache.to_string())], file.data)
        .into_response()
}
