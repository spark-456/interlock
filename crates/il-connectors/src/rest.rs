//! JSON calls to an app's REST API with a bearer key, shared by the API-key connectors.

use il_sdk::prelude::*;
use reqwest::Method;

/// The app's API root; tests point it at a mock server through the `_base` setting.
pub(crate) fn base(ctx: &NodeCtx, default: &str) -> String {
    #[cfg(test)]
    if let Some(base) = ctx.settings["_base"].as_str() {
        return base.to_string();
    }
    let _ = ctx;
    default.to_string()
}

/// The `api_key` saved for `key` on /connections.
pub(crate) async fn api_key(ctx: &NodeCtx, key: &str, app: &str) -> Result<String, NodeError> {
    let credential = ctx.credential(key).await?;
    credential.secret["api_key"]
        .as_str()
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .ok_or_else(|| NodeError::Failed(format!("{app} key is missing; add it on Connections")))
}

/// One request; rate limits and server errors are retryable, other failures carry the app's message.
pub(crate) async fn call(
    ctx: &NodeCtx,
    app: &str,
    method: Method,
    url: &str,
    token: &str,
    headers: &[(&str, &str)],
    body: Option<&Value>,
) -> Result<Value, NodeError> {
    let mut req = ctx.http().request(method, url).map_err(|e| NodeError::Failed(e.to_string()))?.bearer_auth(token);
    for (name, value) in headers {
        req = req.header(*name, *value);
    }
    if let Some(body) = body {
        req = req.json(body);
    }
    let resp = req.send().await.map_err(|_| NodeError::Retryable(format!("{app} request failed")))?;
    let status = resp.status().as_u16();
    let value: Value = resp.json().await.unwrap_or(Value::Null);
    match status {
        200..=299 => Ok(value),
        401 | 403 => Err(NodeError::Failed(format!("{app} rejected the key; check it on Connections"))),
        429 | 500..=599 => Err(NodeError::Retryable(format!("{app} is busy (HTTP {status})"))),
        _ => {
            let message = value["message"]
                .as_str()
                .or_else(|| value["error"]["message"].as_str())
                .or_else(|| value["error"].as_str())
                .unwrap_or("request failed");
            Err(NodeError::Failed(format!("{app}: {message} (HTTP {status})")))
        }
    }
}

/// Runs a send once per node run and item: a retry or a resumed run returns the saved result.
pub(crate) async fn once<F>(ctx: &NodeCtx, key: &str, send: F) -> Result<Value, NodeError>
where
    F: Future<Output = Result<Value, NodeError>>,
{
    let key = format!("{key}:{}", ctx.dedupe_key());
    let store = &ctx.services.store;
    if let Some(saved) = store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
        return Ok(saved);
    }
    let result = send.await?;
    store.kv_put(&key, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
    Ok(result)
}

/// `url` with `segments` appended, each percent-encoded (table names may have spaces).
pub(crate) fn join(url: &str, segments: &[&str]) -> Result<String, NodeError> {
    let mut url = url::Url::parse(url).map_err(|_| NodeError::Failed("invalid API address".into()))?;
    url.path_segments_mut()
        .map_err(|_| NodeError::Failed("invalid API address".into()))?
        .pop_if_empty()
        .extend(segments);
    Ok(url.into())
}

/// A required, non-empty string setting.
pub(crate) fn setting<'a>(ctx: &'a NodeCtx, name: &str) -> Result<&'a str, NodeError> {
    ctx.settings[name]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| NodeError::Failed(format!("{name} is required")))
}

/// An id made only of letters, digits, `-` and `_`, so it can't change the request path.
pub(crate) fn id<'a>(ctx: &'a NodeCtx, name: &str) -> Result<&'a str, NodeError> {
    let value = setting(ctx, name)?;
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
        return Err(NodeError::Failed(format!("{name} is not a valid id")));
    }
    Ok(value)
}
