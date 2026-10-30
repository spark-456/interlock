//! Per-peer fixed-window limits for public ingress.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::connect_info::ConnectInfo;
use axum::extract::{Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use il_core::ApiError;

use crate::wiring::AppState;

struct Bucket {
    start: Instant,
    count: u32,
}

/// One counter per IP and public endpoint class.
#[derive(Default)]
pub struct RateLimiter {
    buckets: Mutex<HashMap<(IpAddr, &'static str), Bucket>>,
}

impl RateLimiter {
    fn allow(&self, ip: IpAddr, class: &'static str, limit: u32, window: Duration) -> bool {
        let Ok(mut buckets) = self.buckets.lock() else { return false };
        let now = Instant::now();
        if buckets.len() >= 4096 && !buckets.contains_key(&(ip, class)) {
            buckets.retain(|_, bucket| now.duration_since(bucket.start) < Duration::from_secs(3600));
            if buckets.len() >= 4096 {
                return false;
            }
        }
        let bucket = buckets.entry((ip, class)).or_insert(Bucket { start: now, count: 0 });
        if now.duration_since(bucket.start) >= window {
            bucket.start = now;
            bucket.count = 0;
        }
        if bucket.count >= limit {
            return false;
        }
        bucket.count += 1;
        true
    }
}

/// The client address our proxy appended to `X-Forwarded-For` (its last entry; earlier ones can be
/// written by the client).
fn client_ip(headers: &axum::http::HeaderMap) -> Option<IpAddr> {
    let value = headers.get("x-forwarded-for")?.to_str().ok()?;
    value.rsplit(',').next()?.trim().parse().ok()
}

/// Applies limits to hooks, published APIs, MCP, widget chat and sign-in.
pub async fn middleware(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let path = request.uri().path();
    let widget_chat = request.method() == Method::POST && path.starts_with("/w/") && path.ends_with("/chat");
    let public = ["/hooks", "/v1", "/mcp", "/api/auth"]
        .iter()
        .any(|prefix| path == *prefix || path.starts_with(&format!("{prefix}/")));
    if !public && !widget_chat {
        return next.run(request).await;
    }
    let peer = request.extensions().get::<ConnectInfo<SocketAddr>>().map(|info| info.0.ip());
    let forwarded = state.config.server.trust_proxy.then(|| client_ip(request.headers())).flatten();
    let ip = forwarded.or(peer).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let limit = state.config.server.public_requests_per_minute;
    if state.rate_limiter.allow(ip, "public", limit, Duration::from_secs(60)) {
        next.run(request).await
    } else {
        (StatusCode::TOO_MANY_REQUESTS, Json(ApiError::new("rate_limited", "Try again later."))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderMap;

    use super::client_ip;

    #[test]
    fn q03_the_proxys_entry_is_the_client() {
        let mut headers = HeaderMap::new();
        assert_eq!(client_ip(&headers), None);
        headers.insert("x-forwarded-for", "6.6.6.6, 203.0.113.9".parse().unwrap());
        assert_eq!(client_ip(&headers), Some("203.0.113.9".parse().unwrap()), "a spoofed first entry is ignored");
        headers.insert("x-forwarded-for", "not an ip".parse().unwrap());
        assert_eq!(client_ip(&headers), None);
    }
}
