//! `trigger.api_call` — API call.
//!
//! API calls arrive through `TriggerSupervisor::invoke` (OpenAI-compatible API, REST).
//! Activation starts nothing in the background.

use il_sdk::prelude::*;

/// API call node.
#[derive(Debug, Default)]
pub struct TriggerApiCall;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.api_call",
        Category::Trigger,
        "API call",
        "Starts the flow from the OpenAI-compatible API or a REST call. Pair with Respond.",
    )
    .icon("code")
    .trigger()
    .status(NodeStatus::Ready)
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "inputSchema": {
                "type": "object",
                "title": "Input schema",
                "description": "JSON Schema of the request body."
            },
            "description": {
                "type": "string",
                "title": "Description"
            }
        }
    }))
    .example(
        "Answer a question",
        json!({
            "description": "Answers support questions."
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerApiCall {
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
impl TriggerNode for TriggerApiCall {
    async fn start(&self, _ctx: il_sdk::TriggerCtx) -> Result<il_sdk::TriggerHandle, NodeError> {
        Ok(il_sdk::TriggerHandle::noop())
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerApiCall)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerApiCall);
    }
}
