//! `ai.generate_voice` — Generate voice.
//!

use il_sdk::prelude::*;

/// Generate voice node.
#[derive(Debug, Default)]
pub struct AiGenerateVoice;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.generate_voice",
        Category::Ai,
        "Generate voice",
        "Reads text aloud as a voiceover. Uses the recorded provider in demo mode and tests.",
    )
    .icon("voice")
    .status(NodeStatus::Ready)
    .cost(CostHint::Medium)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "text"
        ],
        "properties": {
            "text": {
                "type": "string",
                "title": "Text",
                "default": "{{ item.text }}",
                "x-widget": "expression"
            },
            "voice": {
                "type": "string",
                "title": "Voice",
                "default": "Maya",
                "enum": ["Maya", "Arjun", "Serene", "Bernard", "Billy", "Mark", "Mabel", "Eleanor", "Elias", "Lara", "Lisa", "Marlene", "Miriam", "Paula", "Rina"],
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Voiceover",
        json!({
            "text": "{{ item.text }}",
            "voice": "Maya"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiGenerateVoice {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        super::execute(ctx, input, il_sdk::services::MediaKind::Voice, "text").await
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiGenerateVoice)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiGenerateVoice);
    }
}
