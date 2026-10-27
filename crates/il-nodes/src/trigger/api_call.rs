//! `trigger.api_call` — API call.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
