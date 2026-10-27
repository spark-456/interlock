//! `trigger.chat` — Chat message.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

use il_sdk::prelude::*;

/// Chat message node.
#[derive(Debug, Default)]
pub struct TriggerChat;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.chat",
        Category::Trigger,
        "Chat message",
        "Starts the flow for each chat message (playground, widget, chat page). Pair with Respond.",
    )
    .icon("chat")
    .trigger()
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "welcomeMessage": {
                "type": "string",
                "title": "Welcome message"
            },
            "suggestedQuestions": {
                "type": "array",
                "title": "Suggested questions",
                "items": {
                    "type": "string"
                },
                "default": []
            }
        }
    }))
    .example(
        "Support chat",
        json!({
            "welcomeMessage": "Hi, I'm the Larkspur assistant. How can I help?",
            "suggestedQuestions": [
                "Where is my order?"
            ]
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerChat {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerChat)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerChat);
    }
}
