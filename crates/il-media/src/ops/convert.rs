//! `media.convert` — Convert a media file to a common video or audio format.

use il_sdk::prelude::*;

/// Convert media node.
#[derive(Debug, Default)]
pub struct MediaConvert;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.convert",
        Category::Media,
        "Convert media",
        "Converts a video or audio file to MP4, WebM, MP3 or WAV.",
    )
    .icon("repeat")
    .status(NodeStatus::Ready)
    .settings_schema(json!({
        "type":"object", "required":["file","format"],
        "properties":{
            "file":{"type":"string","title":"Media file","x-widget":"file"},
            "format":{"type":"string","title":"Format","enum":["mp4","webm","mp3","wav"],"x-widget":"select"}
        }
    }))
    .example("Convert to MP4", json!({"file":"{{ item.files.video }}","format":"mp4"}))
});

#[async_trait]
impl NodeExecutor for MediaConvert {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = super::ffmpeg::file_id(ctx, &input, "file")?;
        let source = super::ffmpeg::input_path(ctx, &id).await?;
        let duration = super::ffmpeg::duration(&source).await?;
        let format = ctx.settings["format"].as_str().ok_or_else(|| NodeError::Failed("format is required".into()))?;
        let (mime, codecs): (&str, &[&str]) = match format {
            "mp4" => ("video/mp4", &["-c:v", "libx264", "-c:a", "aac", "-movflags", "+faststart"]),
            "webm" => ("video/webm", &["-c:v", "libvpx-vp9", "-c:a", "libopus"]),
            "mp3" => ("audio/mpeg", &["-vn", "-c:a", "libmp3lame"]),
            "wav" => ("audio/wav", &["-vn", "-c:a", "pcm_s16le"]),
            _ => return Err(NodeError::Failed("unsupported format".into())),
        };
        let target = super::ffmpeg::output_path(format);
        let mut args: Vec<std::ffi::OsString> = vec!["-i".into(), source.into_os_string()];
        args.extend(codecs.iter().map(Into::into));
        args.push(target.as_os_str().into());
        super::ffmpeg::run(ctx, &args, Some(duration), None).await?;
        let file = super::ffmpeg::save(ctx, &target, &format!("converted.{format}"), mime).await?;
        Ok(super::ffmpeg::with_file(input, "converted", file))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaConvert)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaConvert);
    }
}
