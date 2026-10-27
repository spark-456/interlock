//! Instagram, TikTok, LinkedIn, X: validate, preview, record dry-run (need platform approval to post).
//!
//! Owned by task C03. Add one file per node and list it here.

pub mod instagram;
pub mod linkedin;
pub mod tiktok;
pub mod x;

#[cfg(test)]
mod tests;

use il_sdk::prelude::*;

pub(super) async fn media(
    ctx: &NodeCtx,
    input: &NodeInput,
    key: &str,
    fallback: &str,
    required: bool,
) -> Result<Option<FileRef>, NodeError> {
    let id = ctx
        .settings
        .get(key)
        .and_then(|value| value.as_str().or_else(|| value["id"].as_str()))
        .filter(|id| !id.is_empty())
        .or_else(|| input.main().first().and_then(|item| item.files.get(fallback).map(|file| file.id.as_str())));
    match id {
        Some(id) => ctx.files().get(id).await.map(|(file, _)| Some(file)).map_err(|e| NodeError::Failed(e.to_string())),
        None if required => Err(NodeError::Failed(format!("{key} file is required"))),
        None => Ok(None),
    }
}

pub(super) fn preview(
    input: NodeInput,
    platform: &str,
    text: &str,
    media: Option<FileRef>,
    options: Value,
) -> NodeOutput {
    let mut item = input.main().first().cloned().unwrap_or_default();
    if !item.json.is_object() {
        item.json = json!({"input": item.json});
    }
    item.json["dryRun"] = json!({
        "status":"dryRun", "platform":platform, "note":"Needs platform approval to post",
        "preview":{"text":text, "media":media, "options":options}
    });
    NodeOutput::out(vec![item])
}
