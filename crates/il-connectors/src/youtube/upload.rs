//! `app.youtube.upload` — Upload video (YouTube).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Upload video (YouTube) node.
#[derive(Debug, Default)]
pub struct AppYoutubeUpload;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.youtube.upload",
        Category::App,
        "Upload video (YouTube)",
        "Uploads a video to YouTube (private by default).",
    )
    .icon("video")
    .app("youtube")
    .external()
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/youtube.upload"],
    ))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video",
            "title"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video",
                "default": "{{ item.files.video }}",
                "x-widget": "file"
            },
            "title": {
                "type": "string",
                "title": "Title",
                "x-widget": "expression"
            },
            "description": {
                "type": "string",
                "title": "Description",
                "x-widget": "longtext"
            },
            "privacy": {
                "type": "string",
                "title": "Privacy",
                "enum": [
                    "private",
                    "unlisted"
                ],
                "default": "private",
                "x-widget": "select"
            },
            "tags": {
                "type": "array",
                "title": "Tags",
                "items": {
                    "type": "string"
                },
                "default": []
            }
        }
    }))
    .example(
        "Private upload",
        json!({
            "video": "{{ item.files.clip_16x9 }}",
            "title": "{{ item.title }}",
            "privacy": "private"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppYoutubeUpload {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppYoutubeUpload)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppYoutubeUpload);
    }
}
