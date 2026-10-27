//! `human.approval` — Approval.
//!
//! Requests an approval and routes its eventual decision.

use il_core::{ApprovalTimeout, Decision};
use il_sdk::ctx::ApprovalSpec;
use il_sdk::prelude::*;

/// Approval node.
#[derive(Debug, Default)]
pub struct HumanApproval;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("human.approval", Category::Human, "Approval", "Pauses until a person approves or rejects (in the app, optionally Slack). The payload can be edited before approving.")
        .icon("check-circle")
        .outputs(&["approved", "rejected"])
        .credential(CredentialSpec { optional: true, ..cred("slack", "slack", CredentialKind::OAuth2, "Slack workspace", &["chat:write"]) })
        .settings_schema(json!({
            "type": "object",
            "required": [
                "title"
            ],
            "properties": {
                "title": {
                    "type": "string",
                    "title": "Title"
                },
                "message": {
                    "type": "string",
                    "title": "Message",
                    "x-widget": "longtext"
                },
                "payload": {
                    "type": "string",
                    "title": "Payload",
                    "default": "{{ item }}",
                    "x-widget": "expression"
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
                },
                "slackChannel": {
                    "type": "string",
                    "title": "Slack channel"
                },
                "timeoutHours": {
                    "type": "number",
                    "title": "Timeout (hours)",
                    "default": 48,
                    "minimum": 0.1
                },
                "onTimeout": {
                    "type": "string",
                    "title": "On timeout",
                    "enum": [
                        "reject",
                        "approve",
                        "keepWaiting"
                    ],
                    "default": "keepWaiting",
                    "x-widget": "select"
                }
            }
        }))
        .example("Approve posts", json!({
            "title": "Approve this week's promo posts",
            "payload": "{{ item }}",
            "channels": [
                "inapp",
                "slack"
            ],
            "slackChannel": "#marketing"
        }))
});

#[async_trait]
impl NodeExecutor for HumanApproval {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        if let Some(decision) = &ctx.resume {
            let mut item = input.main().first().cloned().unwrap_or_default();
            if let Some(payload) = &decision.payload {
                item.json = payload.clone();
            }
            let port = match decision.decision {
                Decision::Approve => "approved",
                Decision::Reject => "rejected",
            };
            return Ok(NodeOutput::port(port, vec![item]));
        }
        let channels = ctx.settings["channels"]
            .as_array()
            .map(|values| values.iter().filter_map(|value| value.as_str().map(str::to_owned)).collect())
            .unwrap_or_else(|| vec!["inapp".into()]);
        let spec = ApprovalSpec {
            title: ctx.settings["title"].as_str().unwrap_or_default().into(),
            message: ctx.settings["message"].as_str().unwrap_or_default().into(),
            payload: ctx
                .settings
                .get("payload")
                .cloned()
                .unwrap_or_else(|| input.main().first().map(|item| item.json.clone()).unwrap_or(Value::Null)),
            channels,
            target: ctx.settings["slackChannel"].as_str().map(str::to_owned),
            timeout_hours: ctx.settings["timeoutHours"].as_f64(),
            on_timeout: match ctx.settings["onTimeout"].as_str() {
                Some("approve") => ApprovalTimeout::Approve,
                Some("reject") => ApprovalTimeout::Reject,
                _ => ApprovalTimeout::KeepWaiting,
            },
            credential_id: ctx.credential_ids.get("slack").cloned(),
        };
        Ok(NodeOutput::waiting(ctx.request_approval(spec).await?))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(HumanApproval)) }

#[cfg(test)]
mod tests {
    use il_core::{ApprovalDecision, ApprovalTimeout, Decision};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::HumanApproval);
    }

    #[tokio::test]
    async fn b08_approval_waits_with_configured_channel_and_timeout() {
        let harness = NodeHarness::new(super::HumanApproval)
            .settings(json!({"title":"Approve post","channels":["inapp","slack"],"slackChannel":"#team","timeoutHours":2.0,"onTimeout":"reject","payload":"{{ item.draft }}"}))
            .input_json(vec![json!({"draft":"Hello"})]);
        assert!(harness.run().await.unwrap().waiting_for.is_some());
        let requests = harness.approvals.requested.lock().unwrap();
        assert_eq!(requests[0].payload, "Hello");
        assert_eq!(requests[0].target.as_deref(), Some("#team"));
        assert_eq!(requests[0].timeout_hours, Some(2.0));
        assert_eq!(requests[0].on_timeout, ApprovalTimeout::Reject);
    }

    #[tokio::test]
    async fn b08_approval_routes_edited_payload() {
        let harness = NodeHarness::new(super::HumanApproval)
            .settings(json!({"title":"Approve"}))
            .input_json(vec![json!({"draft":"Old"})])
            .resume(ApprovalDecision {
                decision: Decision::Approve,
                payload: Some(json!({"draft":"Edited"})),
                comment: None,
            });
        assert_eq!(harness.run().await.unwrap().ports["approved"][0].json["draft"], "Edited");
    }

    #[tokio::test]
    async fn b08_approval_routes_rejection() {
        let harness = NodeHarness::new(super::HumanApproval)
            .settings(json!({"title":"Approve"}))
            .resume(ApprovalDecision { decision: Decision::Reject, payload: None, comment: None });
        assert!(harness.run().await.unwrap().ports.contains_key("rejected"));
    }
}
