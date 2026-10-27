//! `media.cut_clips` — Cut clips.
//!
//! Even cuts use source duration; highlights start around detected scene changes.

use il_sdk::prelude::*;
use std::{path::Path, time::Duration};
use tokio::{process::Command, time::timeout};

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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = super::ffmpeg::file_id(ctx, &input, "video")?;
        let source = super::ffmpeg::input_path(ctx, &id).await?;
        let duration = super::ffmpeg::duration(&source).await?;
        let count = ctx.settings["count"].as_u64().unwrap_or(3) as usize;
        let length = (ctx.settings["durationSec"].as_f64().unwrap_or(15.0)).min(duration);
        let scenes =
            if ctx.settings["strategy"].as_str() == Some("highlights") { scene_changes(&source).await } else { vec![] };
        let mut item = input.main().first().cloned().unwrap_or_default();
        for index in 0..count {
            let even_start = (duration - length).max(0.0) * index as f64 / (count.saturating_sub(1).max(1)) as f64;
            let start = scenes
                .get(index)
                .map(|scene| (scene - length / 2.0).clamp(0.0, (duration - length).max(0.0)))
                .unwrap_or(even_start);
            let target = super::ffmpeg::output_path("mp4");
            let args = vec![
                "-ss".into(),
                start.to_string().into(),
                "-i".into(),
                source.as_os_str().into(),
                "-t".into(),
                length.to_string().into(),
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
            super::ffmpeg::run(ctx, &args, Some(length), None).await?;
            let label = format!("clip_{}", index + 1);
            let file = super::ffmpeg::save(ctx, &target, &format!("{label}.mp4"), "video/mp4").await?;
            item.files.insert(label, file);
        }
        Ok(NodeOutput::out(vec![item]))
    }
}

async fn scene_changes(source: &Path) -> Vec<f64> {
    let result = timeout(
        Duration::from_secs(90),
        Command::new("ffmpeg")
            .args(["-nostdin", "-hide_banner", "-loglevel", "info", "-i"])
            .arg(source)
            .args(["-t", "600", "-vf", "select=gt(scene\\,0.35),showinfo", "-an", "-f", "null", "-"])
            .kill_on_drop(true)
            .output(),
    )
    .await;
    let Ok(Ok(output)) = result else { return vec![] };
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| line.split("pts_time:").nth(1)?.split_whitespace().next()?.parse::<f64>().ok())
        .collect()
}

inventory::submit! { NodeFactory::new(|| Box::new(MediaCutClips)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaCutClips);
    }
}
