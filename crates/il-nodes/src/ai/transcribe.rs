//! `ai.transcribe` — Transcribe audio.
//!
//! Transcribes a stored audio file through the model gateway.

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file_id = ctx.settings["audio"]
            .as_str()
            .filter(|id| !id.starts_with("{{"))
            .or_else(|| input.main().first()?.files.get("audio").map(|file| file.id.as_str()))
            .ok_or_else(|| NodeError::Failed("audio file is required".into()))?;
        let (file, bytes) = ctx.files().get(file_id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        if !file.mime.starts_with("audio/") {
            return Err(NodeError::Failed("file is not audio".into()));
        }
        let transcript = ctx.models().transcribe(&file, &bytes).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["transcript"] = json!(transcript.text);
        item.json["language"] = json!(transcript.language);
        item.json["markAs"] = json!(ctx.settings["markAs"].as_str().unwrap_or("none"));
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiTranscribe)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::files::FileStore;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiTranscribe);
    }

    #[tokio::test]
    async fn b07_transcribe_returns_marked_transcript() {
        let harness = NodeHarness::new(super::AiTranscribe)
            .settings(json!({"audio":"{{ item.files.audio }}","markAs":"instructions"}));
        let file = harness.fakes.files.put("memo.mp3", "audio/mpeg", vec![1, 2, 3].into()).await.unwrap();
        let mut item = il_core::Item::json(json!({}));
        item.files.insert("audio".into(), file);
        let harness = harness.input(vec![item]);
        harness.fakes.models.set_transcript("Use a friendly tone");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["transcript"], "Use a friendly tone");
        assert_eq!(out.ports["out"][0].json["markAs"], "instructions");
    }
}
