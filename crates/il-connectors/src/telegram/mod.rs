//! Telegram: bot send with media.
//!
//! Owned by task C02. Add one file per node and list it here.

pub mod send;

use il_sdk::prelude::*;

pub(super) async fn send(
    ctx: &NodeCtx,
    token: &str,
    chat_id: &str,
    text: &str,
    media: &Value,
) -> Result<String, NodeError> {
    let base = if cfg!(test) {
        ctx.settings["_base"].as_str().unwrap_or("https://api.telegram.org")
    } else {
        "https://api.telegram.org"
    };
    let media_id = media.as_str().or_else(|| media["id"].as_str());
    let (method, request) = if let Some(id) = media_id {
        let (reference, data) = ctx.files().get(id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let (method, field) = if reference.mime.starts_with("image/") {
            ("sendPhoto", "photo")
        } else if reference.mime.starts_with("video/") {
            ("sendVideo", "video")
        } else {
            ("sendDocument", "document")
        };
        let part = reqwest::multipart::Part::bytes(data.to_vec())
            .file_name(reference.name)
            .mime_str(&reference.mime)
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let form = reqwest::multipart::Form::new()
            .text("chat_id", chat_id.to_owned())
            .text("caption", text.to_owned())
            .part(field, part);
        (method, Some(form))
    } else {
        ("sendMessage", None)
    };
    let url = format!("{base}/bot{token}/{method}");
    let builder = ctx.http().post(&url).map_err(|_| NodeError::Failed("invalid Telegram destination".into()))?;
    let response = match request {
        Some(form) => builder.multipart(form).send().await,
        None => builder.json(&json!({"chat_id": chat_id, "text": text})).send().await,
    }
    .map_err(|_| NodeError::Retryable("Telegram request failed".into()))?;
    let status = response.status();
    let value: Value = response.json().await.map_err(|_| NodeError::Failed("invalid Telegram response".into()))?;
    if !status.is_success() || value["ok"] != true {
        return Err(NodeError::Failed(format!(
            "Telegram: {}",
            value["description"].as_str().unwrap_or("request failed")
        )));
    }
    value["result"]["message_id"]
        .as_i64()
        .map(|id| id.to_string())
        .ok_or_else(|| NodeError::Failed("Telegram response missing message id".into()))
}
