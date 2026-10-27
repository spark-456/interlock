//! `media.resize` — Resize for platform.
//!
//! Preserves source proportions by cropping or padding each target aspect.

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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = super::ffmpeg::file_id(ctx, &input, "video")?;
        let source = super::ffmpeg::input_path(ctx, &id).await?;
        let duration = super::ffmpeg::duration(&source).await?;
        let aspects = ctx.settings["aspects"]
            .as_array()
            .cloned()
            .unwrap_or_else(|| vec![json!("9:16"), json!("1:1"), json!("16:9")]);
        let fit = ctx.settings["fit"].as_str().unwrap_or("crop");
        let mut item = input.main().first().cloned().unwrap_or_default();
        for aspect in aspects {
            let (width, height, label) = match aspect.as_str() {
                Some("9:16") => (1080, 1920, "9x16"),
                Some("1:1") => (1080, 1080, "1x1"),
                Some("16:9") => (1920, 1080, "16x9"),
                _ => return Err(NodeError::Failed("invalid aspect".into())),
            };
            let filter = if fit == "pad" {
                format!(
                    "scale={width}:{height}:force_original_aspect_ratio=decrease,pad={width}:{height}:(ow-iw)/2:(oh-ih)/2"
                )
            } else {
                format!("scale={width}:{height}:force_original_aspect_ratio=increase,crop={width}:{height}")
            };
            let target = super::ffmpeg::output_path("mp4");
            let args = vec![
                "-i".into(),
                source.as_os_str().into(),
                "-vf".into(),
                filter.into(),
                "-c:v".into(),
                "libx264".into(),
                "-preset".into(),
                "veryfast".into(),
                "-c:a".into(),
                "aac".into(),
                "-movflags".into(),
                "+faststart".into(),
                target.as_os_str().into(),
            ];
            super::ffmpeg::run(ctx, &args, Some(duration), None).await?;
            let file = super::ffmpeg::save(ctx, &target, &format!("clip_{label}.mp4"), "video/mp4").await?;
            item.files.insert(format!("clip_{label}"), file);
        }
        Ok(NodeOutput::out(vec![item]))
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
