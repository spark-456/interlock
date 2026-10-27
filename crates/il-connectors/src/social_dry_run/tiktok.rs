//! `app.tiktok.post` — Post video (TikTok).
//!
//! Validates a video and returns a dry-run preview.

use il_sdk::prelude::*;

/// Post video (TikTok) node.
#[derive(Debug, Default)]
pub struct AppTiktokPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.tiktok.post",
        Category::App,
        "Post video (TikTok)",
        "Previews a TikTok video; needs platform approval to post.",
    )
    .icon("send")
    .app("tiktok")
    .external()
    .status(NodeStatus::DryRun)
    .credential(CredentialSpec {
        optional: true,
        ..cred("tiktok", "tiktok", CredentialKind::OAuth2, "TikTok account", &[])
    })
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video (9:16)",
                "default": "{{ item.files.clip_9x16 }}",
                "x-widget": "file"
            },
            "caption": {
                "type": "string",
                "title": "Caption",
                "x-widget": "longtext"
            },
            "privacy": {
                "type": "string",
                "title": "Privacy",
                "enum": [
                    "public",
                    "friends",
                    "private"
                ],
                "default": "public",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Preview",
        json!({
            "video": "{{ item.files.clip_9x16 }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppTiktokPost {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file = super::media(ctx, &input, "video", "clip_9x16", true)
            .await?
            .ok_or_else(|| NodeError::Failed("video is required".into()))?;
        if !matches!(file.mime.as_str(), "video/mp4" | "video/webm" | "video/quicktime") {
            return Err(NodeError::Failed("TikTok video must be MP4, WebM or MOV".into()));
        }
        let caption = ctx.settings["caption"].as_str().unwrap_or("");
        if caption.encode_utf16().count() > 2200 {
            return Err(NodeError::Failed("TikTok caption exceeds 2200 UTF-16 characters".into()));
        }
        let privacy = ctx.settings["privacy"].as_str().unwrap_or("public");
        Ok(super::preview(input, "tiktok", caption, Some(file), json!({"privacy":privacy})))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppTiktokPost)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppTiktokPost);
    }
}
