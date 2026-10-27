//! `app.instagram.post_reel` — Post reel (Instagram).
//!
//! Validates a reel and returns a dry-run preview.

use il_sdk::prelude::*;

/// Post reel (Instagram) node.
#[derive(Debug, Default)]
pub struct AppInstagramPostReel;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.instagram.post_reel",
        Category::App,
        "Post reel (Instagram)",
        "Previews an Instagram reel; needs platform approval to post.",
    )
    .icon("send")
    .app("instagram")
    .external()
    .status(NodeStatus::DryRun)
    .credential(CredentialSpec {
        optional: true,
        ..cred("instagram", "instagram", CredentialKind::OAuth2, "Instagram account", &[])
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
            "coverAtSec": {
                "type": "number",
                "title": "Cover at (s)",
                "default": 1.0,
                "minimum": 0.0
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
impl NodeExecutor for AppInstagramPostReel {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file = super::media(ctx, &input, "video", "clip_9x16", true)
            .await?
            .ok_or_else(|| NodeError::Failed("video is required".into()))?;
        if !file.mime.starts_with("video/") {
            return Err(NodeError::Failed("Instagram reel requires a video file".into()));
        }
        let caption = ctx.settings["caption"].as_str().unwrap_or("");
        let cover = ctx.settings["coverAtSec"].as_f64().unwrap_or(1.0);
        Ok(super::preview(input, "instagram", caption, Some(file), json!({"coverAtSec":cover})))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppInstagramPostReel)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppInstagramPostReel);
    }
}
