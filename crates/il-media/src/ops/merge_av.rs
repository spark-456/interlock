//! `media.merge_av` — Merge audio and video.
//!
//! Replaces or mixes the video's audio track with another file.

use il_sdk::prelude::*;
use std::{path::Path, time::Duration};
use tokio::{process::Command, time::timeout};

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let video = super::ffmpeg::input_path(ctx, &super::ffmpeg::file_id(ctx, &input, "video")?).await?;
        let audio = super::ffmpeg::input_path(ctx, &super::ffmpeg::file_id(ctx, &input, "audio")?).await?;
        let duration = super::ffmpeg::duration(&video).await?;
        let volume = ctx.settings["volume"].as_f64().unwrap_or(1.0);
        let mix = ctx.settings["mode"].as_str() == Some("mix") && has_audio(&video).await;
        let target = super::ffmpeg::output_path("mp4");
        let mut args: Vec<std::ffi::OsString> =
            vec!["-i".into(), video.into_os_string(), "-i".into(), audio.into_os_string()];
        if mix {
            args.extend([
                "-filter_complex".into(),
                format!("[1:a]volume={volume}[voice];[0:a][voice]amix=inputs=2:duration=first[a]").into(),
                "-map".into(),
                "0:v:0".into(),
                "-map".into(),
                "[a]".into(),
            ]);
        } else {
            args.extend([
                "-map".into(),
                "0:v:0".into(),
                "-map".into(),
                "1:a:0".into(),
                "-filter:a".into(),
                format!("volume={volume}").into(),
            ]);
        }
        args.extend([
            "-c:v".into(),
            "copy".into(),
            "-c:a".into(),
            "aac".into(),
            "-shortest".into(),
            target.as_os_str().into(),
        ]);
        super::ffmpeg::run(ctx, &args, Some(duration), None).await?;
        let file = super::ffmpeg::save(ctx, &target, "merged.mp4", "video/mp4").await?;
        Ok(super::ffmpeg::with_file(input, "merged", file))
    }
}

async fn has_audio(path: &Path) -> bool {
    timeout(
        Duration::from_secs(15),
        Command::new("ffprobe")
            .args(["-v", "error", "-select_streams", "a", "-show_entries", "stream=index", "-of", "csv=p=0"])
            .arg(path)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .ok()
    .and_then(Result::ok)
    .is_some_and(|output| output.status.success() && !output.stdout.is_empty())
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaMergeAv)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaMergeAv);
    }
}
