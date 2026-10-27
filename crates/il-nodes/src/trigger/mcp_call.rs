//! `trigger.mcp_call` — MCP call.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

use il_sdk::prelude::*;

/// MCP call node.
#[derive(Debug, Default)]
pub struct TriggerMcpCall;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.mcp_call",
        Category::Trigger,
        "MCP call",
        "Exposes the flow as an MCP tool; each call starts a run. Pair with Respond.",
    )
    .icon("plug")
    .trigger()
    .settings_schema(json!({
        "type": "object",
        "required": [
            "toolName"
        ],
        "properties": {
            "toolName": {
                "type": "string",
                "title": "Tool name"
            },
            "toolDescription": {
                "type": "string",
                "title": "Tool description"
            },
            "inputSchema": {
                "type": "object",
                "title": "Input schema"
            }
        }
    }))
    .example(
        "Ask support",
        json!({
            "toolName": "ask_larkspur",
            "toolDescription": "Ask the Larkspur support assistant."
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerMcpCall {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerMcpCall)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerMcpCall);
    }
}
