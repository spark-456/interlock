//! `human.ask` — Ask a question.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B08 implements it.

use il_sdk::prelude::*;

/// Ask a question node.
#[derive(Debug, Default)]
pub struct HumanAsk;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "human.ask",
        Category::Human,
        "Ask a question",
        "Pauses and asks a person for missing information, then continues with the answers.",
    )
    .icon("help")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "question"
        ],
        "properties": {
            "question": {
                "type": "string",
                "title": "Question",
                "x-widget": "longtext"
            },
            "fields": {
                "type": "array",
                "title": "Answer fields",
                "items": {
                    "type": "object",
                    "title": "Field",
                    "properties": {
                        "name": {
                            "type": "string",
                            "title": "Name"
                        },
                        "label": {
                            "type": "string",
                            "title": "Label"
                        }
                    },
                    "required": [
                        "name"
                    ]
                },
                "default": []
            },
            "channels": {
                "type": "array",
                "title": "Ask in",
                "items": {
                    "type": "string",
                    "enum": [
                        "inapp",
                        "slack"
                    ]
                },
                "default": [
                    "inapp"
                ]
            }
        }
    }))
    .example(
        "Missing order number",
        json!({
            "question": "Which order is this about?",
            "fields": [
                {
                    "name": "orderNumber",
                    "label": "Order number"
                }
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for HumanAsk {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(HumanAsk)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::HumanAsk);
    }
}
