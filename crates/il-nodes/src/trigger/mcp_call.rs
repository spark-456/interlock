//! `trigger.mcp_call` — MCP call.
//!
//! MCP tool calls arrive through `TriggerSupervisor::invoke`.
//! Activation starts nothing in the background.

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        // The engine emits a trigger's items as its output; executing it again passes them through.
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for TriggerMcpCall {
    async fn start(&self, _ctx: il_sdk::TriggerCtx) -> Result<il_sdk::TriggerHandle, NodeError> {
        Ok(il_sdk::TriggerHandle::noop())
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
