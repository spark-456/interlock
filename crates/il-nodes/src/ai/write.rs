//! `ai.write` — Write.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B07 implements it.

use il_sdk::prelude::*;

/// Write node.
#[derive(Debug, Default)]
pub struct AiWrite;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "ai.write",
        Category::Ai,
        "Write",
        "Writes text from instructions and input: scripts, replies, posts, summaries in a given tone.",
    )
    .icon("pen")
    .cost(CostHint::Low)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "instructions"
        ],
        "properties": {
            "instructions": {
                "type": "string",
                "title": "Instructions",
                "x-widget": "longtext"
            },
            "input": {
                "type": "string",
                "title": "Input",
                "default": "{{ item }}",
                "x-widget": "expression"
            },
            "tone": {
                "type": "string",
                "title": "Tone",
                "enum": [
                    "neutral",
                    "friendly",
                    "upbeat",
                    "formal"
                ],
                "default": "friendly",
                "x-widget": "select"
            },
            "length": {
                "type": "string",
                "title": "Length",
                "enum": [
                    "short",
                    "medium",
                    "long"
                ],
                "default": "medium",
                "x-widget": "select"
            },
            "outputField": {
                "type": "string",
                "title": "Output field",
                "default": "text"
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
        "Promo video script",
        json!({
            "instructions": "Write a 20-second upbeat promo script for this product.",
            "input": "{{ item.title }}: {{ item.description }}",
            "tone": "upbeat",
            "length": "short"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AiWrite {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AiWrite)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AiWrite);
    }
}
