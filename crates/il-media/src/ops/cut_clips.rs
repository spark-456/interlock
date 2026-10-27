//! `media.cut_clips` — Cut clips.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M01 implements it.

use il_sdk::prelude::*;

/// Cut clips node.
#[derive(Debug, Default)]
pub struct MediaCutClips;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.cut_clips",
        Category::Media,
        "Cut clips",
        "Cuts a video into short clips (highlights or even segments).",
    )
    .icon("scissors")
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
            "count": {
                "type": "integer",
                "title": "Clips",
                "default": 3,
                "minimum": 1,
                "maximum": 10
            },
            "durationSec": {
                "type": "integer",
                "title": "Clip length (s)",
                "default": 15,
                "minimum": 3,
                "maximum": 90
            },
            "strategy": {
                "type": "string",
                "title": "Strategy",
                "enum": [
                    "highlights",
                    "even"
                ],
                "default": "even",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Three 15 s clips",
        json!({
            "video": "{{ item.files.video }}",
            "count": 3,
            "durationSec": 15
        }),
    )
});

#[async_trait]
impl NodeExecutor for MediaCutClips {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaCutClips)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaCutClips);
    }
}
