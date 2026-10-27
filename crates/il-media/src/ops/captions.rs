//! `media.captions` — Add captions.
//!
//! Writes caption text to a temporary file so ffmpeg never parses user text as a filter.

use il_sdk::prelude::*;
use std::path::PathBuf;

struct Cue {
    timing: Option<(f64, f64)>,
    text: String,
}

fn installed_font() -> Option<PathBuf> {
    std::env::var_os("IL_MEDIA_FONT_FILE").map(PathBuf::from).filter(|path| path.is_file()).or_else(|| {
        [
            "C:/Windows/Fonts/arial.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/System/Library/Fonts/Supplemental/Arial.ttf",
        ]
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
    })
}

fn timecode(value: &str) -> Option<f64> {
    let mut parts = value.trim().split(':');
    let hours = parts.next()?.parse::<f64>().ok()?;
    let minutes = parts.next()?.parse::<f64>().ok()?;
    let seconds = parts.next()?.replace(',', ".").parse::<f64>().ok()?;
    Some(hours * 3600.0 + minutes * 60.0 + seconds)
}

fn cues(text: &str) -> Result<Vec<Cue>, NodeError> {
    if !text.contains("-->") {
        return Ok(vec![Cue { timing: None, text: text.to_owned() }]);
    }
    text.replace("\r\n", "\n")
        .split("\n\n")
        .filter(|block| !block.trim().is_empty())
        .map(|block| {
            let mut lines = block.lines();
            let first = lines.next().ok_or_else(|| NodeError::Failed("invalid SRT cue".into()))?;
            let timing = if first.trim().chars().all(|c| c.is_ascii_digit()) { lines.next() } else { Some(first) }
                .ok_or_else(|| NodeError::Failed("SRT cue time missing".into()))?;
            let (start, end) =
                timing.split_once("-->").ok_or_else(|| NodeError::Failed("invalid SRT cue time".into()))?;
            let start = timecode(start).ok_or_else(|| NodeError::Failed("invalid SRT start time".into()))?;
            let end = timecode(end).ok_or_else(|| NodeError::Failed("invalid SRT end time".into()))?;
            let caption = lines.collect::<Vec<_>>().join("\n");
            if !start.is_finite() || !end.is_finite() || end <= start || caption.trim().is_empty() {
                return Err(NodeError::Failed("invalid SRT cue".into()));
            }
            Ok(Cue { timing: Some((start, end)), text: caption })
        })
        .collect()
}

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = super::ffmpeg::file_id(ctx, &input, "video")?;
        let source = super::ffmpeg::input_path(ctx, &id).await?;
        let duration = super::ffmpeg::duration(&source).await?;
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("caption text is required".into()))?;
        let work = std::env::temp_dir().join(il_sdk::id("caption"));
        tokio::fs::create_dir(&work).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let result = async {
            let font = installed_font().ok_or_else(|| NodeError::Failed("no caption font found; set IL_MEDIA_FONT_FILE".into()))?;
            tokio::fs::copy(font, work.join("caption-font.ttf")).await.map_err(|e| NodeError::Failed(e.to_string()))?;
            let position = match ctx.settings["position"].as_str().unwrap_or("bottom") {
                "top" => "40", "center" => "(h-text_h)/2", _ => "h-text_h-40",
            };
            let size = if ctx.settings["style"].as_str() == Some("clean") { 36 } else { 48 };
            let mut filters = Vec::new();
            for (index, cue) in cues(text)?.into_iter().enumerate() {
                let name = format!("caption-{index}.txt");
                tokio::fs::write(work.join(&name), cue.text).await.map_err(|e| NodeError::Failed(e.to_string()))?;
                let enable = cue.timing.map(|(start, end)| format!(":enable='between(t,{start},{end})'")).unwrap_or_default();
                filters.push(format!("drawtext=fontfile=caption-font.ttf:textfile={name}:fontcolor=white:fontsize={size}:box=1:boxcolor=black@0.6:x=(w-text_w)/2:y={position}{enable}"));
            }
            let filter = filters.join(",");
            let target = work.join("captioned.mp4");
            let args = vec![
                "-i".into(), source.into_os_string(), "-vf".into(), filter.into(),
                "-c:v".into(), "libx264".into(), "-preset".into(), "veryfast".into(),
                "-c:a".into(), "aac".into(), "-movflags".into(), "+faststart".into(), target.as_os_str().into(),
            ];
            super::ffmpeg::run(ctx, &args, Some(duration), Some(&work)).await?;
            super::ffmpeg::save(ctx, &target, "captioned.mp4", "video/mp4").await
        }.await;
        let _ = tokio::fs::remove_dir_all(&work).await;
        Ok(super::ffmpeg::with_file(input, "captioned", result?))
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
