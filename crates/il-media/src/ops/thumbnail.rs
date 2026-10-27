//! `media.thumbnail` — Thumbnail.
//!
//! Runs ffmpeg to extract one frame.

use il_sdk::prelude::*;

/// Thumbnail node.
#[derive(Debug, Default)]
pub struct MediaThumbnail;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("media.thumbnail", Category::Media, "Thumbnail", "Grabs a frame from a video as an image.")
        .icon("frame")
        .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = super::ffmpeg::file_id(ctx, &input, "video")?;
        let source = super::ffmpeg::input_path(ctx, &id).await?;
        let at = ctx.settings["atSec"].as_f64().unwrap_or(1.0);
        let width = ctx.settings["width"].as_u64().unwrap_or(1280);
        let target = super::ffmpeg::output_path("jpg");
        let args = vec![
            "-ss".into(),
            at.to_string().into(),
            "-i".into(),
            source.into_os_string(),
            "-frames:v".into(),
            "1".into(),
            "-vf".into(),
            format!("scale={width}:-2").into(),
            "-q:v".into(),
            "2".into(),
            target.as_os_str().into(),
        ];
        super::ffmpeg::run(ctx, &args, None, None).await?;
        let file = super::ffmpeg::save(ctx, &target, "thumbnail.jpg", "image/jpeg").await?;
        Ok(super::ffmpeg::with_file(input, "thumbnail", file))
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
