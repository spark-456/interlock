//! `ai.describe_image` — Describe image.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiDescribeImage)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiDescribeImage);
    }
}
