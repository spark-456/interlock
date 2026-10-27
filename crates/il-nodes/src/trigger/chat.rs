//! `trigger.chat` — Chat message.
//!
//! Chat messages arrive through `TriggerSupervisor::invoke` (playground, widget, chat page).
//! Activation starts nothing in the background.

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        // The engine emits a trigger's items as its output; executing it again passes them through.
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerChat {
    async fn start(&self, _ctx: il_sdk::TriggerCtx) -> Result<il_sdk::TriggerHandle, NodeError> {
        Ok(il_sdk::TriggerHandle::noop())
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
