//! `ai.transcribe` — Transcribe audio.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

/// Transcribe audio node.
#[derive(Debug, Default)]
pub struct AiTranscribe;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.transcribe",
        Category::Ai,
        "Transcribe audio",
        "Turns speech into text, optionally marked as knowledge or instructions.",
    )
    .icon("mic")
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "audio"
        ],
        "properties": {
            "audio": {
                "type": "string",
                "title": "Audio",
                "default": "{{ item.files.audio }}",
                "x-widget": "file"
            },
            "language": {
                "type": "string",
                "title": "Language",
                "description": "Leave empty to detect."
            },
            "markAs": {
                "type": "string",
                "title": "Use as",
                "enum": [
                    "none",
                    "knowledge",
                    "instructions"
                ],
                "default": "none",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Voice memo",
        json!({
            "audio": "{{ item.files.audio }}",
            "markAs": "instructions"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiTranscribe {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiTranscribe)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiTranscribe);
    }
}
