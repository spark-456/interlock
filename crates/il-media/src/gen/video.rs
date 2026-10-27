//! `ai.generate_video` — Generate video.
//!

use il_sdk::prelude::*;

/// Generate video node.
#[derive(Debug, Default)]
pub struct AiGenerateVideo;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.generate_video",
        Category::Ai,
        "Generate video",
        "Generates a short video from a script or prompt. Uses the recorded provider in demo mode and tests.",
    )
    .icon("film")
    .status(NodeStatus::Ready)
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
});

#[async_trait]
impl NodeExecutor for AiGenerateVideo {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        super::execute(ctx, input, il_sdk::services::MediaKind::Video, "prompt").await
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiGenerateVideo)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiGenerateVideo);
    }
}
