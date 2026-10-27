//! Gmail request and MIME helpers shared by its nodes.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use il_sdk::{NodeError, ctx::NodeCtx};
use serde_json::{Value, json};

pub fn endpoint(ctx: &NodeCtx, path: &str) -> String {
    #[cfg(test)]
    let base = ctx.settings["_base"].as_str().unwrap_or("https://gmail.googleapis.com");
    #[cfg(not(test))]
    let base = {
        let _ = ctx;
        "https://gmail.googleapis.com"
    };
    format!("{base}/gmail/v1/users/me/{path}")
}

pub async fn token(ctx: &NodeCtx) -> Result<String, NodeError> {
    let credential = ctx.credential("google").await?;
    credential.secret["access_token"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| NodeError::Failed("Google credential has no access token".into()))
}

pub fn header(value: &str) -> Result<&str, NodeError> {
    if value.contains(['\r', '\n']) {
        return Err(NodeError::Failed("invalid mail header".into()));
    }
    Ok(value)
}

pub fn id(value: &str) -> Result<&str, NodeError> {
    if value.is_empty() || !value.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
        return Err(NodeError::Failed("invalid Gmail id".into()));
    }
    Ok(value)
}

pub fn raw(to: &str, subject: &str, body: &str, extra: &[(&str, &str)]) -> Result<String, NodeError> {
    header(to)?;
    header(subject)?;
    let mut message =
        format!("To: {to}\r\nSubject: {subject}\r\nMIME-Version: 1.0\r\nContent-Type: text/plain; charset=UTF-8\r\n");
    for (name, value) in extra {
        message.push_str(&format!("{}: {}\r\n", header(name)?, header(value)?));
    }
    message.push_str("\r\n");
    message.push_str(body);
    Ok(URL_SAFE_NO_PAD.encode(message))
}

pub async fn send(ctx: &NodeCtx, raw: String, thread_id: Option<&str>) -> Result<Value, NodeError> {
    let key = format!("gmail:send:{}", ctx.dedupe_key());
    if let Some(saved) = ctx.services.store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
        return Ok(saved);
    }
    let token = token(ctx).await?;
    let mut body = json!({"raw":raw});
    if let Some(thread_id) = thread_id {
        body["threadId"] = json!(thread_id);
    }
    let response = ctx
        .http()
        .post(&endpoint(ctx, "messages/send"))
        .map_err(|e| NodeError::Failed(e.to_string()))?
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|e| NodeError::Retryable(e.to_string()))?
        .error_for_status()
        .map_err(|e| NodeError::Failed(e.to_string()))?;
    let result: Value = response.json().await.map_err(|e| NodeError::Failed(e.to_string()))?;
    ctx.services.store.kv_put(&key, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
    Ok(result)
}
