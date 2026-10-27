//! `ai.describe_image` — Describe image.
//!
//! Sends stored image bytes to the model gateway.

use super::common;
use base64::{Engine, engine::general_purpose::STANDARD};
use il_sdk::prelude::*;
use il_sdk::services::models::{ChatMessage, ChatRequest, ModelRole};

const PROMPT: &str = include_str!("../../../../prompts/nodes/describe_image.md");

/// Describe image node.
#[derive(Debug, Default)]
pub struct AiDescribeImage;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.describe_image",
        Category::Ai,
        "Describe image",
        "Describes an image or answers a question about it.",
    )
    .icon("eye")
    .status(NodeStatus::Ready)
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "image"
        ],
        "properties": {
            "image": {
                "type": "string",
                "title": "Image",
                "default": "{{ item.files.image }}",
                "x-widget": "file"
            },
            "prompt": {
                "type": "string",
                "title": "Question",
                "default": "Describe this image."
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
        "Alt text",
        json!({
            "image": "{{ item.files.image }}",
            "prompt": "Write short alt text."
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiDescribeImage {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file_id = ctx.settings["image"]
            .as_str()
            .filter(|id| !id.starts_with("{{"))
            .or_else(|| input.main().first()?.files.get("image").map(|file| file.id.as_str()))
            .ok_or_else(|| NodeError::Failed("image file is required".into()))?;
        let (file, bytes) = ctx.files().get(file_id).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        if !file.mime.starts_with("image/") {
            return Err(NodeError::Failed("file is not an image".into()));
        }
        let mut user = ChatMessage::user(ctx.settings["prompt"].as_str().unwrap_or("Describe this image."));
        user.image_url = Some(format!("data:{};base64,{}", file.mime, STANDARD.encode(bytes)));
        let response = ctx
            .models()
            .chat(
                common::role(&ctx.settings, ModelRole::Agent)?,
                ChatRequest { messages: vec![ChatMessage::system(PROMPT.trim()), user], ..Default::default() },
            )
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        Ok(common::output(&input, "description", json!(response.content)))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiDescribeImage)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::files::FileStore;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiDescribeImage);
    }

    #[tokio::test]
    async fn b07_describe_image_sends_stored_image() {
        let harness = NodeHarness::new(super::AiDescribeImage)
            .settings(json!({"image":"{{ item.files.image }}","prompt":"What is shown?"}));
        let file = harness.fakes.files.put("photo.png", "image/png", vec![1, 2, 3].into()).await.unwrap();
        let mut item = il_core::Item::json(json!({}));
        item.files.insert("image".into(), file);
        let harness = harness.input(vec![item]);
        harness.fakes.models.push_text("A cat");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["description"], "A cat");
        assert!(harness.fakes.models.calls()[0].1.messages[1].image_url.as_deref().unwrap().contains("AQID"));
    }
}
