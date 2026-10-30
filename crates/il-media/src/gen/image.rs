//! `ai.generate_image` — Generate image.
//!

use il_sdk::prelude::*;

/// Generate image node.
#[derive(Debug, Default)]
pub struct AiGenerateImage;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.generate_image",
        Category::Ai,
        "Generate image",
        "Generates an image from a prompt. Test runs use sample media.",
    )
    .icon("image")
    .status(NodeStatus::Ready)
    .credential(super::own_key())
    .cost(CostHint::Medium)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "prompt"
        ],
        "properties": {
            "prompt": {
                "type": "string",
                "title": "Prompt",
                "x-widget": "longtext"
            },
            "size": {
                "type": "string",
                "title": "Size",
                "enum": [
                    "1024x1024",
                    "1024x1792",
                    "1792x1024"
                ],
                "default": "1024x1024",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Product hero",
        json!({
            "prompt": "A ceramic mug on a linen table, soft morning light",
            "size": "1024x1024"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiGenerateImage {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        super::execute(ctx, input, il_sdk::services::MediaKind::Image, "prompt").await
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiGenerateImage)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiGenerateImage);
    }
}
