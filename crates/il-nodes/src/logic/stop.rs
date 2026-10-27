//! `logic.stop` — Stop.
//!

use il_sdk::prelude::*;

/// Stop node.
#[derive(Debug, Default)]
pub struct LogicStop;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.stop", Category::Logic, "Stop", "Ends this branch, as success or with an error message.")
        .icon("stop")
        .status(NodeStatus::Ready)
        .outputs(&[])
        .settings_schema(json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "title": "Status",
                    "enum": [
                        "success",
                        "error"
                    ],
                    "default": "success",
                    "x-widget": "select"
                },
                "reason": {
                    "type": "string",
                    "title": "Reason"
                }
            }
        }))
        .example(
            "Ignore spam",
            json!({
                "status": "success",
                "reason": "Spam"
            }),
        )
});

#[async_trait]
impl NodeExecutor for LogicStop {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        if ctx.settings["status"].as_str().unwrap_or("success") == "error" {
            return Err(NodeError::Failed(ctx.settings["reason"].as_str().unwrap_or("Stopped by flow").into()));
        }
        Ok(NodeOutput::default())
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicStop)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicStop);
    }

    #[tokio::test]
    async fn b06_stop_ends_branch_or_fails() {
        let success = NodeHarness::new(super::LogicStop).settings(json!({"status":"success"})).run().await.unwrap();
        assert!(success.ports.is_empty());
        let error = NodeHarness::new(super::LogicStop)
            .settings(json!({"status":"error","reason":"bad input"}))
            .run()
            .await
            .unwrap_err();
        assert!(error.to_string().contains("bad input"));
    }
}
