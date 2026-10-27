//! `logic.wait` — Wait.
//!

use std::time::Duration;

use il_sdk::prelude::*;

/// Wait node.
#[derive(Debug, Default)]
pub struct LogicWait;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("logic.wait", Category::Logic, "Wait", "Pauses for a duration or until a time.")
        .icon("timer")
        .status(NodeStatus::Ready)
        .settings_schema(json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "title": "Mode",
                    "enum": [
                        "duration",
                        "until"
                    ],
                    "default": "duration",
                    "x-widget": "select"
                },
                "seconds": {
                    "type": "integer",
                    "title": "Seconds",
                    "default": 60,
                    "minimum": 1
                },
                "until": {
                    "type": "string",
                    "title": "Until",
                    "x-widget": "expression"
                }
            }
        }))
        .example(
            "One minute",
            json!({
                "mode": "duration",
                "seconds": 60
            }),
        )
});

#[async_trait]
impl NodeExecutor for LogicWait {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let duration = match ctx.settings["mode"].as_str().unwrap_or("duration") {
            "duration" => Duration::from_secs(ctx.settings["seconds"].as_u64().unwrap_or(60)),
            "until" => {
                let until = ctx.settings["until"]
                    .as_str()
                    .ok_or_else(|| NodeError::Failed("until must be an RFC 3339 time".into()))?;
                let until = chrono::DateTime::parse_from_rfc3339(until)
                    .map_err(|_| NodeError::Failed("until must be an RFC 3339 time".into()))?;
                until.signed_duration_since(ctx.services.clock.now()).to_std().unwrap_or_default()
            }
            _ => return Err(NodeError::Failed("unknown wait mode".into())),
        };
        if !duration.is_zero() {
            tokio::select! {
                _ = ctx.services.clock.sleep(duration) => {},
                _ = ctx.cancel.cancelled() => return Err(NodeError::Cancelled),
            }
        }
        Ok(NodeOutput::out(input.main().to_vec()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicWait)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicWait);
    }

    #[tokio::test]
    async fn b06_wait_until_past_time_keeps_input() {
        let output = NodeHarness::new(super::LogicWait)
            .settings(json!({"mode":"until","until":"2020-01-01T00:00:00Z"}))
            .input_json(vec![json!({"n":1})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"][0].json["n"], 1);
    }
}
