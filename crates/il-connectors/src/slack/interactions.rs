//! Slack interactive approval callback. Verifies Slack's signed request before deciding.

use axum::{
    Router,
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use il_core::{ApprovalDecision, Decision};
use il_sdk::registry::{RouteCtx, RouteFactory};
use serde_json::Value;
use sha2::Sha256;

/// Routes for Slack interactive messages.
pub fn routes(ctx: &RouteCtx) -> Router {
    Router::new().route("/api/slack/actions", post(decide)).with_state(ctx.clone())
}

inventory::submit! { RouteFactory { name: "slack-actions", make: routes } }

fn hex_decode(input: &str) -> Option<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        return None;
    }
    input
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let hi = (pair[0] as char).to_digit(16)?;
            let lo = (pair[1] as char).to_digit(16)?;
            Some(((hi << 4) | lo) as u8)
        })
        .collect()
}

fn valid_signature(headers: &HeaderMap, body: &[u8], secret: &str, now: i64) -> bool {
    let Some(timestamp) = headers.get("x-slack-request-timestamp").and_then(|v| v.to_str().ok()) else { return false };
    let Ok(seconds) = timestamp.parse::<i64>() else { return false };
    if (now - seconds).abs() > 300 {
        return false;
    }
    let Some(signature) =
        headers.get("x-slack-signature").and_then(|v| v.to_str().ok()).and_then(|v| v.strip_prefix("v0="))
    else {
        return false;
    };
    let Some(signature) = hex_decode(signature) else { return false };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else { return false };
    mac.update(b"v0:");
    mac.update(timestamp.as_bytes());
    mac.update(b":");
    mac.update(body);
    mac.verify_slice(&signature).is_ok()
}

async fn decide(State(ctx): State<RouteCtx>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let Ok(secret) = std::env::var("SLACK_SIGNING_SECRET") else { return StatusCode::SERVICE_UNAVAILABLE };
    if !valid_signature(&headers, &body, &secret, Utc::now().timestamp()) {
        return StatusCode::UNAUTHORIZED;
    }
    let Some(payload) = url::form_urlencoded::parse(&body)
        .find(|(key, _)| key == "payload")
        .and_then(|(_, value)| serde_json::from_str::<Value>(&value).ok())
    else {
        return StatusCode::BAD_REQUEST;
    };
    let Some(action) = payload["actions"].as_array().and_then(|a| a.first()) else { return StatusCode::BAD_REQUEST };
    let decision = match action["action_id"].as_str() {
        Some("approve") => Decision::Approve,
        Some("reject") => Decision::Reject,
        _ => return StatusCode::BAD_REQUEST,
    };
    let Some(id) = action["value"].as_str() else { return StatusCode::BAD_REQUEST };
    let by = payload["user"]["id"].as_str().map(str::to_owned);
    match ctx.runner.decide(id, ApprovalDecision { decision, payload: None, comment: None }, by).await {
        Ok(()) => StatusCode::OK,
        Err(_) => StatusCode::CONFLICT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn c02_slack_rejects_forged_and_stale_callbacks() {
        let mut headers = HeaderMap::new();
        headers.insert("x-slack-request-timestamp", HeaderValue::from_static("1000"));
        assert!(!valid_signature(&headers, b"payload=x", "secret", 1000));
        let mut mac = Hmac::<Sha256>::new_from_slice(b"secret").unwrap();
        mac.update(b"v0:1000:payload=x");
        let sig = mac.finalize().into_bytes().iter().map(|b| format!("{b:02x}")).collect::<String>();
        headers.insert("x-slack-signature", HeaderValue::from_str(&format!("v0={sig}")).unwrap());
        assert!(valid_signature(&headers, b"payload=x", "secret", 1000));
        assert!(!valid_signature(&headers, b"payload=y", "secret", 1000));
        assert!(!valid_signature(&headers, b"payload=x", "secret", 1301));
    }
}
