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

fn cues_from(text: &str) -> Result<Vec<Cue>, NodeError> {
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

/// Words per caption, so long transcript segments become readable lines.
const WORDS_PER_CUE: usize = 6;

/// Cues from transcript segments (`{start, end, text}`), split into short lines timed by their length.
/// `{{ item.segments }}` renders to the list, or to its JSON inside longer text.
fn timed_cues(value: &Value) -> Result<Vec<Cue>, NodeError> {
    let parsed;
    let list = match value {
        Value::Array(list) => list,
        Value::String(text) if text.trim_start().starts_with('[') => {
            parsed = serde_json::from_str::<Vec<Value>>(text)
                .map_err(|e| NodeError::Failed(format!("timed segments are not a list: {e}")))?;
            &parsed
        }
        _ => return Ok(Vec::new()),
    };
    let mut cues = Vec::new();
    for segment in list {
        let (Some(start), Some(end), Some(text)) =
            (segment["start"].as_f64(), segment["end"].as_f64(), segment["text"].as_str())
        else {
            return Err(NodeError::Failed("each timed segment needs start, end and text".into()));
        };
        let words: Vec<&str> = text.split_whitespace().collect();
        // Whisper hears "." or "..." in silence; those are not captions.
        let spoken = text.chars().any(char::is_alphanumeric);
        if !spoken || !start.is_finite() || !end.is_finite() || end <= start {
            continue;
        }
        // A line ends at a sentence end or after WORDS_PER_CUE words.
        let mut lines: Vec<String> = Vec::new();
        let mut count = 0;
        for word in words {
            match lines.last_mut() {
                Some(line) if count > 0 => {
                    line.push(' ');
                    line.push_str(word);
                }
                _ => lines.push(word.to_owned()),
            }
            count += 1;
            if count == WORDS_PER_CUE || word.ends_with(['.', '!', '?']) {
                count = 0;
            }
        }
        let chars: usize = lines.iter().map(|line| line.chars().count()).sum();
        let mut at = start;
        for line in lines {
            let until = at + (end - start) * line.chars().count() as f64 / chars as f64;
            cues.push(Cue { timing: Some((at, until)), text: line });
            at = until;
        }
    }
    Ok(cues)
}

/// Add captions node.
#[derive(Debug, Default)]
pub struct MediaCaptions;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "media.captions",
        Category::Media,
        "Add captions",
        "Burns captions into a video from text, SRT, or timed segments from Transcribe audio so they follow the voiceover.",
    )
    .icon("captions")
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
            "text": {
                "type": "string",
                "title": "Captions (text or SRT)",
                "x-widget": "expression"
            },
            "segments": {
                "type": "string",
                "title": "Timed segments",
                "description": "Optional. {{ item.segments }} from Transcribe audio on the voiceover; used instead of the text.",
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
    .example(
        "Follow the voiceover",
        json!({
            "video": "{{ item.files.video }}",
            "segments": "{{ item.segments }}"
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
        let mut cues = timed_cues(&ctx.settings["segments"])?;
        if cues.is_empty() {
            let text = ctx.settings["text"].as_str().filter(|t| !t.trim().is_empty());
            cues = cues_from(
                text.ok_or_else(|| NodeError::Failed("caption text or timed segments are required".into()))?,
            )?;
        }
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
            for (index, cue) in cues.into_iter().enumerate() {
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
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::MediaCaptions);
    }

    #[test]
    fn timed_segments_become_short_lines_that_follow_the_voice() {
        let segments = json!([
            {"start": 0.0, "end": 4.0, "text": "Meet the Ember mug. It keeps your coffee hot"},
            {"start": 4.5, "end": 5.0, "text": " . "},
            {"start": 5.0, "end": 6.5, "text": "for three hours."}
        ]);
        let cues = super::timed_cues(&json!(segments.to_string())).unwrap();
        let lines: Vec<_> = cues.iter().map(|c| (c.text.as_str(), c.timing.unwrap())).collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].0, "Meet the Ember mug.");
        assert_eq!(lines[1].0, "It keeps your coffee hot");
        assert_eq!(lines[0].1.0, 0.0);
        assert!((lines[0].1.1 - lines[1].1.0).abs() < 1e-9 && lines[0].1.1 > 1.5);
        assert!((lines[1].1.1 - 4.0).abs() < 1e-9);
        assert_eq!(lines[2], ("for three hours.", (5.0, 6.5)));
        assert!(super::timed_cues(&json!("")).unwrap().is_empty());
        assert!(super::timed_cues(&json!([{"text": "no times"}])).is_err());
    }
}
