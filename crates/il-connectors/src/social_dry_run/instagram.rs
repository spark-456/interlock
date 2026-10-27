//! `app.instagram.post_reel` — Post reel (Instagram).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C03 implements it.

use il_sdk::prelude::*;

/// Post reel (Instagram) node.
#[derive(Debug, Default)]
pub struct AppInstagramPostReel;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.instagram.post_reel",
        Category::App,
        "Post reel (Instagram)",
        "Prepares a post reel post. Dry-run: validates and previews the post; needs platform approval to post.",
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
