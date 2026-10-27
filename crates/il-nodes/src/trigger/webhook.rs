//! `trigger.webhook` — Webhook.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B05 implements it.

use il_sdk::prelude::*;

/// Webhook node.
#[derive(Debug, Default)]
pub struct TriggerWebhook;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "trigger.webhook",
        Category::Trigger,
        "Webhook",
        "Starts the flow when an HTTP request hits the flow's secret webhook URL.",
    )
    .icon("webhook")
    .trigger()
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "method": {
                "type": "string",
                "title": "Method",
                "enum": [
                    "POST",
                    "GET",
                    "PUT"
                ],
                "default": "POST",
                "x-widget": "select"
            },
            "respond": {
                "type": "string",
                "title": "Respond",
                "enum": [
                    "immediately",
                    "lastNode"
                ],
                "default": "immediately",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Receive a POST",
        json!({
            "method": "POST",
            "respond": "immediately"
        }),
    )
});

#[async_trait]
impl NodeExecutor for TriggerWebhook {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(TriggerWebhook)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::TriggerWebhook);
    }
}
