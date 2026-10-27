//! `human.ask` — Ask a question.
//!
//! Pauses for an answer to a configured question.

use il_core::Decision;
use il_sdk::ctx::ApprovalSpec;
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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        if let Some(decision) = &ctx.resume {
            if decision.decision == Decision::Reject {
                return Err(NodeError::Failed("question was declined".into()));
            }
            let mut item = input.main().first().cloned().unwrap_or_default();
            if !item.json.is_object() {
                item.json = json!({});
            }
            item.json["answers"] = decision.payload.clone().unwrap_or(Value::Null);
            return Ok(NodeOutput::out(vec![item]));
        }
        let question = ctx.settings["question"].as_str().unwrap_or_default();
        let channels = ctx.settings["channels"]
            .as_array()
            .map(|values| values.iter().filter_map(|value| value.as_str().map(str::to_owned)).collect())
            .unwrap_or_else(|| vec!["inapp".into()]);
        let spec = ApprovalSpec {
            title: question.into(),
            message: question.into(),
            payload: json!({"fields":ctx.settings["fields"]}),
            channels,
            target: None,
            timeout_hours: None,
        };
        Ok(NodeOutput::waiting(ctx.request_approval(spec).await?))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(HumanAsk)) }

#[cfg(test)]
mod tests {
    use il_core::{ApprovalDecision, Decision};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::HumanAsk);
    }

    #[tokio::test]
    async fn b08_ask_waits_with_answer_fields() {
        let harness = NodeHarness::new(super::HumanAsk)
            .settings(json!({"question":"Order number?","fields":[{"name":"orderNumber"}]}));
        assert!(harness.run().await.unwrap().waiting_for.is_some());
        assert_eq!(harness.approvals.requested.lock().unwrap()[0].payload["fields"][0]["name"], "orderNumber");
    }

    #[tokio::test]
    async fn b08_ask_returns_answer() {
        let harness =
            NodeHarness::new(super::HumanAsk).settings(json!({"question":"Order number?"})).resume(ApprovalDecision {
                decision: Decision::Approve,
                payload: Some(json!({"orderNumber":"A12"})),
                comment: None,
            });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["answers"]["orderNumber"], "A12");
    }
}
