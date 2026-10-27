//! `ai.generate_image` — Generate image.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task M02 implements it.

use il_sdk::prelude::*;

/// Generate image node.
#[derive(Debug, Default)]
pub struct AiGenerateImage;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.generate_image",
        Category::Ai,
        "Generate image",
        "Generates an image from a prompt. Uses the recorded provider in demo mode and tests.",
    )
    .icon("image")
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
            },
            "style": {
                "type": "string",
                "title": "Style"
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
