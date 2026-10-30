//! `ai.generate_video` — Generate video.
//!
//! With a shot list (from `ai.shot_list`) it generates each shot and joins them into one video.

use il_sdk::prelude::*;
use il_sdk::services::MediaKind;

/// Shots per video; each one is a paid generation.
const MAX_SHOTS: usize = 6;

/// Generate video node.
#[derive(Debug, Default)]
pub struct AiGenerateVideo;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.generate_video",
        Category::Ai,
        "Generate video",
        "Generates a short video from a prompt, or one clip per shot from a shot list, joined in order. \
         Test runs use sample media.",
    )
    .icon("film")
    .status(NodeStatus::Ready)
    .credential(super::own_key())
    .cost(CostHint::High)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "prompt"
        ],
        "properties": {
            "prompt": {
                "type": "string",
                "title": "Script or prompt",
                "default": "{{ item.text }}",
                "x-widget": "longtext"
            },
            "shots": {
                "type": "string",
                "title": "Shot list",
                "description": "Optional. Shots from Plan shots ({{ item.shots }}); each becomes a clip of its own length.",
                "x-widget": "expression"
            },
            "durationSec": {
                "type": "integer",
                "title": "Duration (s)",
                "default": 5,
                "minimum": 2,
                "maximum": 10
            },
            "aspect": {
                "type": "string",
                "title": "Aspect",
                "enum": [
                    "16:9",
                    "9:16"
                ],
                "default": "16:9",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Promo clip",
        json!({
            "prompt": "{{ item.text }}",
            "durationSec": 5,
            "aspect": "16:9"
        }),
    )
    .example(
        "Promo from a shot list",
        json!({
            "prompt": "{{ item.text }}",
            "shots": "{{ item.shots }}",
            "aspect": "16:9"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiGenerateVideo {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let shots = shots(&ctx.settings["shots"])?;
        if shots.is_empty() {
            return super::execute(ctx, input, MediaKind::Video, "prompt").await;
        }
        let (mut clips, mut cost, mut provider) = (Vec::new(), 0.0, String::new());
        for (index, (prompt, duration)) in shots.iter().enumerate() {
            let pct = index as f32 / shots.len() as f32 * 90.0;
            ctx.emit_progress(pct, &format!("Generating shot {} of {}", index + 1, shots.len()));
            let mut params = ctx.settings.clone();
            params["durationSec"] = json!(duration);
            let result = super::generate(ctx, MediaKind::Video, prompt.clone(), params).await?;
            cost += result.cost_usd;
            provider = result.provider;
            clips.push(result.file);
        }
        let file = match clips.as_slice() {
            [one] => one.clone(),
            _ => {
                let aspect = ctx.settings["aspect"].as_str().unwrap_or("16:9");
                crate::ops::join::join_clips(ctx, &clips, aspect).await?
            }
        };
        ctx.emit_progress(100.0, "Generated media saved");
        Ok(super::output(input, MediaKind::Video, file, &provider, cost))
    }
}

/// (prompt, seconds) per shot. `{{ item.shots }}` renders to the list, or to its JSON inside longer text.
fn shots(value: &Value) -> Result<Vec<(String, u64)>, NodeError> {
    let parsed;
    let list = match value {
        Value::Array(list) => list,
        Value::String(text) if text.trim_start().starts_with('[') => {
            parsed = serde_json::from_str::<Vec<Value>>(text)
                .map_err(|e| NodeError::Failed(format!("shot list is not a list: {e}")))?;
            &parsed
        }
        _ => return Ok(Vec::new()),
    };
    if list.len() > MAX_SHOTS {
        return Err(NodeError::Failed(format!("at most {MAX_SHOTS} shots per video")));
    }
    list.iter()
        .map(|shot| {
            let prompt = shot["prompt"]
                .as_str()
                .or(shot.as_str())
                .map(str::trim)
                .filter(|p| !p.is_empty())
                .ok_or_else(|| NodeError::Failed("every shot needs a prompt".into()))?;
            // Runway clips are 2–10 seconds.
            Ok((prompt.to_owned(), shot["durationSec"].as_u64().unwrap_or(5).clamp(2, 10)))
        })
        .collect()
}

inventory::submit! { NodeFactory::new(|| Box::new(AiGenerateVideo)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiGenerateVideo);
    }

    #[test]
    fn shots_come_from_a_list_or_its_json_and_stay_within_limits() {
        let list = json!([{"prompt": " A mug ", "durationSec": 30}, {"prompt": "Close-up", "durationSec": 1}, "Logo"]);
        let expected = vec![("A mug".to_owned(), 10), ("Close-up".to_owned(), 2), ("Logo".to_owned(), 5)];
        assert_eq!(super::shots(&list).unwrap(), expected);
        assert_eq!(super::shots(&json!(list.to_string())).unwrap(), expected);
        assert!(super::shots(&json!("")).unwrap().is_empty());
        assert!(super::shots(&json!([{"durationSec": 3}])).is_err());
        assert!(super::shots(&json!(vec!["shot"; 7])).is_err());
    }

    #[tokio::test]
    async fn one_shot_skips_joining_and_reports_its_cost() {
        let harness = NodeHarness::new(super::AiGenerateVideo)
            .settings(json!({"prompt": "unused", "shots": "{{ item.shots }}"}))
            .input_json(vec![json!({"shots": [{"prompt": "A mug", "durationSec": 4}]})]);
        let result = harness.run().await.unwrap();
        assert_eq!(result.ports["out"][0].files["video"].mime, "video/mp4");
        assert_eq!(result.ports["out"][0].json["video"]["provider"], "fake");
    }
}
