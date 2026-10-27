//! Bluesky: post with media via app password.
//!
//! Owned by task C02. Add one file per node and list it here.

pub mod post;

use chrono::Utc;
use il_sdk::prelude::*;

async fn checked_json(response: reqwest::Response, operation: &str) -> Result<Value, NodeError> {
    if !response.status().is_success() {
        return Err(NodeError::Failed(format!("Bluesky {operation} HTTP {}", response.status())));
    }
    response.json().await.map_err(|_| NodeError::Failed(format!("invalid Bluesky {operation} response")))
}

pub(super) async fn post(
    ctx: &NodeCtx,
    pds: &str,
    identifier: &str,
    password: &str,
    text: &str,
    media: &Value,
    alt: &str,
) -> Result<String, NodeError> {
    let pds_url = url::Url::parse(pds).map_err(|_| NodeError::Failed("invalid Bluesky PDS URL".into()))?;
    if !cfg!(test) && pds_url.scheme() != "https" {
        return Err(NodeError::Failed("Bluesky PDS must use HTTPS".into()));
    }
    let session = ctx
        .http()
        .post(&format!("{pds}/xrpc/com.atproto.server.createSession"))
        .map_err(|e| NodeError::Failed(e.to_string()))?
        .json(&json!({"identifier": identifier, "password": password}))
        .send()
        .await
        .map_err(|_| NodeError::Retryable("Bluesky login request failed".into()))?;
    let session = checked_json(session, "login").await?;
    let token = session["accessJwt"].as_str().ok_or_else(|| NodeError::Failed("Bluesky login token missing".into()))?;
    let did = session["did"].as_str().ok_or_else(|| NodeError::Failed("Bluesky account DID missing".into()))?;
    let mut record = json!({"$type":"app.bsky.feed.post", "text":text, "createdAt":Utc::now().to_rfc3339()});
    if let Some(id) = media.as_str().or_else(|| media["id"].as_str()) {
        let (reference, bytes) = ctx.files().get(id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        if !reference.mime.starts_with("image/") && !reference.mime.starts_with("video/") {
            return Err(NodeError::Failed("Bluesky media must be an image or video".into()));
        }
        let response = ctx
            .http()
            .post(&format!("{pds}/xrpc/com.atproto.repo.uploadBlob"))
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .header(reqwest::header::CONTENT_TYPE, &reference.mime)
            .body(bytes)
            .send()
            .await
            .map_err(|_| NodeError::Retryable("Bluesky media upload failed".into()))?;
        let upload = checked_json(response, "upload").await?;
        let blob = upload.get("blob").ok_or_else(|| NodeError::Failed("Bluesky blob missing".into()))?;
        record["embed"] = if reference.mime.starts_with("video/") {
            json!({"$type":"app.bsky.embed.video", "video":blob, "alt":alt})
        } else {
            json!({"$type":"app.bsky.embed.images", "images":[{"image":blob,"alt":alt}]})
        };
    }
    let response = ctx
        .http()
        .post(&format!("{pds}/xrpc/com.atproto.repo.createRecord"))
        .map_err(|e| NodeError::Failed(e.to_string()))?
        .bearer_auth(token)
        .json(&json!({"repo":did,"collection":"app.bsky.feed.post","record":record}))
        .send()
        .await
        .map_err(|_| NodeError::Retryable("Bluesky post request failed".into()))?;
    let result = checked_json(response, "post").await?;
    result["uri"].as_str().map(str::to_owned).ok_or_else(|| NodeError::Failed("Bluesky post URI missing".into()))
}
