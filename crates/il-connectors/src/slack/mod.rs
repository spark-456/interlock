//! Slack: send message, approval card + decision callback, `Notifier` adapter.
//!
//! Owned by task C02. Add one file per node and list it here.

pub mod approval;
mod interactions;
pub mod send;
pub use interactions::routes;
mod notifier;
pub use notifier::MessagingNotifier;

use il_sdk::prelude::*;

pub(super) fn api_base(settings: &Value) -> String {
    #[cfg(test)]
    if let Some(base) = settings["_base"].as_str() {
        return base.to_owned();
    }
    let _ = settings;
    "https://slack.com/api".into()
}

pub(super) async fn post_message(
    http: &il_sdk::SafeHttp,
    base: &str,
    token: &str,
    channel: &str,
    text: &str,
    blocks: Option<Value>,
) -> Result<Value, NodeError> {
    let mut body = json!({"channel": channel, "text": text});
    if let Some(blocks) = blocks {
        body["blocks"] = blocks;
    }
    let response = http
        .post(&format!("{base}/chat.postMessage"))
        .map_err(|e| NodeError::Failed(e.to_string()))?
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|e| NodeError::Retryable(e.to_string()))?;
    let status = response.status();
    let value: Value = response.json().await.map_err(|e| NodeError::Failed(e.to_string()))?;
    if !status.is_success() || value["ok"] != true {
        return Err(NodeError::Failed(format!("Slack: {}", value["error"].as_str().unwrap_or("request failed"))));
    }
    Ok(value)
}
