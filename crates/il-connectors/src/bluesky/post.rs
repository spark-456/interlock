//! `app.bluesky.post` — Post (Bluesky).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C02 implements it.

use il_sdk::prelude::*;

/// Post (Bluesky) node.
#[derive(Debug, Default)]
pub struct AppBlueskyPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.bluesky.post",
        Category::App,
        "Post (Bluesky)",
        "Posts text with an optional image or video to Bluesky.",
    )
    .icon("send")
    .app("bluesky")
    .external()
    .credential(cred("bluesky", "bluesky", CredentialKind::AppPassword, "Bluesky app password", &[]))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "text"
        ],
        "properties": {
            "text": {
                "type": "string",
                "title": "Post",
                "x-widget": "longtext"
            },
            "media": {
                "type": "string",
                "title": "Media",
                "x-widget": "expression"
            },
            "altText": {
                "type": "string",
                "title": "Alt text"
            }
        }
    }))
    .example(
        "Promo post",
        json!({
            "text": "{{ item.post }}",
            "media": "{{ item.files.clip_1x1 }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppBlueskyPost {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppBlueskyPost)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppBlueskyPost);
    }
}
