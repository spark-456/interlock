//! `media.captions` — Add captions.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M01 implements it.

use il_sdk::prelude::*;

/// Add captions node.
#[derive(Debug, Default)]
pub struct MediaCaptions;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.captions",
        Category::Media,
        "Add captions",
        "Burns captions into a video from text or SRT.",
    )
    .icon("captions")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "video",
            "text"
        ],
        "properties": {
            "video": {
                "type": "string",
                "title": "Video",
                "default": "{{ item.files.video }}",
                "x-widget": "file"
            },
            "text": {
                "type": "string",
                "title": "Captions (text or SRT)",
                "x-widget": "expression"
            },
            "position": {
                "type": "string",
                "title": "Position",
                "enum": [
                    "bottom",
                    "center",
                    "top"
                ],
                "default": "bottom",
                "x-widget": "select"
            },
            "style": {
                "type": "string",
                "title": "Style",
                "enum": [
                    "clean",
                    "bold"
                ],
                "default": "bold",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Burn script",
        json!({
            "video": "{{ item.files.video }}",
            "text": "{{ item.script }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for MediaCaptions {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaCaptions)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaCaptions);
    }
}
