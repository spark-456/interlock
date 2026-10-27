//! `app.tiktok.post` — Post video (TikTok).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C03 implements it.

use il_sdk::prelude::*;

/// Post video (TikTok) node.
#[derive(Debug, Default)]
pub struct AppTiktokPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.tiktok.post",
        Category::App,
        "Post video (TikTok)",
        "Prepares a post video post. Dry-run: validates and previews the post; needs platform approval to post.",
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
