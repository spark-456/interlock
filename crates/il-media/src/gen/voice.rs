//! `ai.generate_voice` — Generate voice.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M02 implements it.

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
                "default": "warm"
            },
            "speed": {
                "type": "number",
                "title": "Speed",
                "default": 1.0,
                "minimum": 0.5,
                "maximum": 2.0
            }
        }
    }))
    .example(
        "Voiceover",
        json!({
            "text": "{{ item.text }}",
            "voice": "warm"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiGenerateVoice {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
