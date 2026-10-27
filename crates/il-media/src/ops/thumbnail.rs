//! `media.thumbnail` — Thumbnail.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M01 implements it.

use il_sdk::prelude::*;

/// Thumbnail node.
#[derive(Debug, Default)]
pub struct MediaThumbnail;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("media.thumbnail", Category::Media, "Thumbnail", "Grabs a frame from a video as an image.")
        .icon("frame")
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
                "atSec": {
                    "type": "number",
                    "title": "At (s)",
                    "default": 1.0,
                    "minimum": 0.0
                },
                "width": {
                    "type": "integer",
                    "title": "Width",
                    "default": 1280,
                    "minimum": 64,
                    "maximum": 3840
                }
            }
        }))
        .example(
            "Cover frame",
            json!({
                "video": "{{ item.files.video }}",
                "atSec": 2
            }),
        )
});

#[async_trait]
impl NodeExecutor for MediaThumbnail {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaThumbnail)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaThumbnail);
    }
}
