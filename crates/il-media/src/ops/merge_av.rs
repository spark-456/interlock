//! `media.merge_av` — Merge audio and video.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M01 implements it.

use il_sdk::prelude::*;

/// Merge audio and video node.
#[derive(Debug, Default)]
pub struct MediaMergeAv;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.merge_av",
        Category::Media,
        "Merge audio and video",
        "Puts a voiceover or music track onto a video.",
    )
    .icon("merge")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video",
            "audio"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video",
                "default": "{{ item.files.video }}",
                "x-widget": "file"
            },
            "audio": {
                "type": "string",
                "title": "Audio",
                "default": "{{ item.files.audio }}",
                "x-widget": "file"
            },
            "mode": {
                "type": "string",
                "title": "Mode",
                "enum": [
                    "replace",
                    "mix"
                ],
                "default": "replace",
                "x-widget": "select"
            },
            "volume": {
                "type": "number",
                "title": "Audio volume",
                "default": 1.0,
                "minimum": 0.0,
                "maximum": 2.0
            }
        }
    }))
    .example(
        "Add voiceover",
        json!({
            "video": "{{ item.files.video }}",
            "audio": "{{ item.files.audio }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for MediaMergeAv {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaMergeAv)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaMergeAv);
    }
}
