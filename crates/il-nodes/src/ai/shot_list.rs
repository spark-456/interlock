//! `ai.shot_list` — Plan shots.
//!
//! The model plans the shots; the prompts are assembled here so the subject, style and lighting
//! read the same in every shot and every shot fits the provider's limits.

use super::common;
use il_sdk::prelude::*;
use il_sdk::services::ModelRole;

const PROMPT: &str = include_str!("../../../../prompts/nodes/shot_list.md");
/// Runway clips are 2–10 seconds; `ai.generate_video` takes at most 6 shots.
const SHOT_SEC: (u64, u64) = (2, 10);
const MAX_SHOTS: u64 = 6;
/// Runway's prompt limit, in characters.
const MAX_PROMPT: usize = 1000;

/// Plan shots node.
#[derive(Debug, Default)]
pub struct AiShotList;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.shot_list",
        Category::Ai,
        "Plan shots",
        "Turns a script or product description into a shot list for Generate video: one prompt per shot \
         with the same subject, style and lighting, a camera move, and a 2–10 s duration.",
    )
    .icon("clapperboard")
    .status(NodeStatus::Ready)
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "string",
                "title": "Script or description",
                "default": "{{ item.text }}",
                "x-widget": "expression"
            },
            "shots": {
                "type": "integer",
                "title": "Shots",
                "default": 3,
                "minimum": 1,
                "maximum": MAX_SHOTS
            },
            "totalSec": {
                "type": "integer",
                "title": "Video length (s)",
                "default": 15,
                "minimum": 2,
                "maximum": 60
            },
            "style": {
                "type": "string",
                "title": "Look and feel",
                "description": "Optional, e.g. bright studio product ad."
            },
            "model": {
                "type": "string",
                "title": "Model",
                "enum": [
                    "architect",
                    "agent",
                    "fast"
                ],
                "default": "agent",
                "x-widget": "model-role"
            }
        }
    }))
    .example(
        "Promo shots",
        json!({
            "input": "{{ item.text }}",
            "shots": 3,
            "totalSec": 15,
            "style": "bright studio product ad"
        }),
    )
});

fn plan_schema(shots: u64) -> Value {
    let text = json!({"type": "string"});
    json!({
        "type": "object",
        "required": ["subject", "style", "lighting", "shots"],
        "additionalProperties": false,
        "properties": {
            "subject": text, "style": text, "lighting": text,
            "shots": {
                "type": "array", "minItems": 1, "maxItems": shots,
                "items": {
                    "type": "object",
                    "required": ["action", "camera", "durationSec"],
                    "additionalProperties": false,
                    "properties": {"action": text, "camera": text, "durationSec": {"type": "integer"}}
                }
            }
        }
    })
}

/// One shot per planned shot, each prompt carrying the shared subject, style and lighting.
fn shots(plan: &Value, count: u64) -> Result<Vec<Value>, NodeError> {
    let field = |value: &Value, key: &str| value[key].as_str().map(str::trim).unwrap_or_default().to_owned();
    let (subject, style, lighting) = (field(plan, "subject"), field(plan, "style"), field(plan, "lighting"));
    let planned = plan["shots"].as_array().filter(|s| !s.is_empty());
    let planned = planned.ok_or_else(|| NodeError::Failed("the model planned no shots".into()))?;
    Ok(planned
        .iter()
        .take(count as usize)
        .map(|shot| {
            let (action, camera) = (field(shot, "action"), field(shot, "camera"));
            let duration = shot["durationSec"].as_u64().unwrap_or(5).clamp(SHOT_SEC.0, SHOT_SEC.1);
            let parts = [("", &camera), ("", &subject), ("", &action), ("Style: ", &style), ("Lighting: ", &lighting)];
            let prompt = parts
                .iter()
                .filter(|(_, text)| !text.is_empty())
                .map(|(label, text)| format!("{label}{}.", text.trim_end_matches('.')))
                .collect::<Vec<_>>()
                .join(" ");
            json!({
                "subject": subject, "action": action, "camera": camera, "style": style, "lighting": lighting,
                "durationSec": duration,
                "prompt": prompt.chars().take(MAX_PROMPT).collect::<String>(),
            })
        })
        .collect())
}

#[async_trait]
impl NodeExecutor for AiShotList {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let count = ctx.settings["shots"].as_u64().unwrap_or(3).clamp(1, MAX_SHOTS);
        let total = ctx.settings["totalSec"].as_u64().unwrap_or(15);
        let mut system = format!(
            "{}\nShots: at most {count}. Target length: {total} seconds. Each shot: {}–{} seconds.",
            PROMPT.trim(),
            SHOT_SEC.0,
            SHOT_SEC.1
        );
        if let Some(style) = ctx.settings["style"].as_str().filter(|s| !s.trim().is_empty()) {
            system.push_str(&format!("\nLook and feel asked for: {}", style.trim()));
        }
        let user = common::input_text(&ctx.settings, &input);
        let response = common::complete(ctx, ModelRole::Agent, system, user, Some(plan_schema(count))).await?;
        let plan = response.json().map_err(|e| NodeError::Failed(e.to_string()))?;
        Ok(common::output(&input, "shots", Value::Array(shots(&plan, count)?)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiShotList)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiShotList);
    }

    #[tokio::test]
    async fn shots_share_subject_style_and_lighting_and_fit_runway() {
        let harness = NodeHarness::new(super::AiShotList)
            .settings(json!({"input": "{{ item.text }}", "shots": 2, "totalSec": 12, "style": "warm and cosy"}))
            .input_json(vec![json!({"text": "Meet the Ember mug: keeps coffee hot for 3 hours."})]);
        harness.fakes.models.push_json(json!({
            "subject": "a matte black ceramic smart mug with a thin copper ring",
            "style": "cosy lifestyle commercial, shallow depth of field.",
            "lighting": "warm morning window light",
            "shots": [
                {"action": "Steam rises from the mug on a wooden table.", "camera": "slow push-in", "durationSec": 14},
                {"action": "A hand lifts the mug.", "camera": "close-up, static", "durationSec": 1},
                {"action": "An extra shot over the limit.", "camera": "wide", "durationSec": 5}
            ]
        }));
        let result = harness.run().await.unwrap();
        let item = &result.ports["out"][0].json;
        assert_eq!(item["text"], "Meet the Ember mug: keeps coffee hot for 3 hours.");
        let shots = item["shots"].as_array().unwrap();
        assert_eq!(shots.len(), 2);
        assert_eq!(shots[0]["durationSec"], 10);
        assert_eq!(shots[1]["durationSec"], 2);
        assert_eq!(
            shots[0]["prompt"],
            "slow push-in. a matte black ceramic smart mug with a thin copper ring. Steam rises from the mug on a \
             wooden table. Style: cosy lifestyle commercial, shallow depth of field. Lighting: warm morning window light."
        );
        for shot in shots {
            assert!(shot["prompt"].as_str().unwrap().contains("matte black ceramic smart mug"));
        }
        let (_, request) = &harness.fakes.models.calls()[0];
        assert!(request.messages[0].content.contains("Look and feel asked for: warm and cosy"));
        assert_eq!(request.json_schema.as_ref().unwrap()["properties"]["shots"]["maxItems"], 2);
    }
}
