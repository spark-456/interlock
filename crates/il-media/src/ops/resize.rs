//! `media.resize` — Resize for platform.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M01 implements it.

use il_sdk::prelude::*;

/// Resize for platform node.
#[derive(Debug, Default)]
pub struct MediaResize;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.resize",
        Category::Media,
        "Resize for platform",
        "Makes 9:16, 1:1 and 16:9 versions with smart crop or padding.",
    )
    .icon("crop")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video",
                "default": "{{ item.files.video }}",
                "x-widget": "file"
            },
            "aspects": {
                "type": "array",
                "title": "Aspects",
                "items": {
                    "type": "string",
                    "enum": [
                        "9:16",
                        "1:1",
                        "16:9"
                    ]
                },
                "default": [
                    "9:16",
                    "1:1",
                    "16:9"
                ]
            },
            "fit": {
                "type": "string",
                "title": "Fit",
                "enum": [
                    "crop",
                    "pad"
                ],
                "default": "crop",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "All platforms",
        json!({
            "video": "{{ item.files.video }}",
            "aspects": [
                "9:16",
                "1:1",
                "16:9"
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for MediaResize {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaResize)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaResize);
    }
}
