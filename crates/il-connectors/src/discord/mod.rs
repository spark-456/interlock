//! Discord: webhook send with attachments.
//!
//! Owned by task C02. Add one file per node and list it here.

pub mod send;

use il_sdk::prelude::*;

pub(super) async fn send(
    ctx: &NodeCtx,
    endpoint: &str,
    bot_token: Option<&str>,
    content: &str,
    attachments: &Value,
) -> Result<String, NodeError> {
    let url = url::Url::parse(endpoint).map_err(|_| NodeError::Failed("invalid Discord destination".into()))?;
    if !cfg!(test)
        && (url.scheme() != "https"
            || url.host_str() != Some("discord.com")
            || !(url.path().starts_with("/api/webhooks/") && bot_token.is_none()
                || url.path().starts_with("/api/v10/channels/") && bot_token.is_some()))
    {
        return Err(NodeError::Failed("invalid Discord destination".into()));
    }
    let mut url = url;
    if bot_token.is_none() {
        url.query_pairs_mut().append_pair("wait", "true");
    }
    let ids: Vec<&str> = match attachments {
        Value::Null => vec![],
        Value::String(id) => vec![id],
        Value::Object(map) => map.get("id").and_then(Value::as_str).into_iter().collect(),
        Value::Array(list) => list.iter().filter_map(|entry| entry.as_str().or_else(|| entry["id"].as_str())).collect(),
        _ => return Err(NodeError::Failed("invalid Discord attachments".into())),
    };
    let mut request =
        ctx.http().post(url.as_str()).map_err(|_| NodeError::Failed("invalid Discord destination".into()))?;
    if let Some(token) = bot_token {
        request = request.header(reqwest::header::AUTHORIZATION, format!("Bot {token}"));
    }
    let response = if ids.is_empty() {
        request.json(&json!({"content": content})).send().await
    } else {
        let mut form = reqwest::multipart::Form::new().text("payload_json", json!({"content": content}).to_string());
        for (index, id) in ids.into_iter().enumerate() {
            let (reference, data) = ctx.files().get(id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
            let part = reqwest::multipart::Part::bytes(data.to_vec())
                .file_name(reference.name)
                .mime_str(&reference.mime)
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            form = form.part(format!("files[{index}]"), part);
        }
        request.multipart(form).send().await
    }
    .map_err(|_| NodeError::Retryable("Discord request failed".into()))?;
    if !response.status().is_success() {
        return Err(NodeError::Failed(format!("Discord HTTP {}", response.status())));
    }
    let value: Value = response.json().await.map_err(|e| NodeError::Failed(e.to_string()))?;
    value["id"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| NodeError::Failed("Discord response missing message id".into()))
}
