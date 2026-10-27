//! `app.slack.approval` — Approval (Slack).
//!
//! Decisions arrive through the signed Slack interaction route.

use il_core::Decision;
use il_sdk::ctx::ApprovalSpec;
use il_sdk::prelude::*;

/// Approval (Slack) node.
#[derive(Debug, Default)]
pub struct AppSlackApproval;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.slack.approval",
        Category::App,
        "Approval (Slack)",
        "Posts an approval card with Approve/Reject buttons and waits for a decision.",
    )
    .icon("check-circle")
    .status(NodeStatus::Ready)
    .app("slack")
    .outputs(&["approved", "rejected"])
    .external()
    .credential(cred("slack", "slack", CredentialKind::OAuth2, "Slack workspace", &["chat:write"]))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "channel",
            "title"
        ],
        "properties": {
            "channel": {
                "type": "string",
                "title": "Channel"
            },
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
            }
        }
    }))
    .example(
        "Refund approval",
        json!({
            "channel": "#support",
            "title": "Send this refund reply?",
            "payload": "{{ item }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppSlackApproval {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        if let Some(decision) = &ctx.resume {
            let port = if decision.decision == Decision::Approve { "approved" } else { "rejected" };
            let mut items = input.main().to_vec();
            if let (Some(payload), Some(item)) = (&decision.payload, items.first_mut()) {
                item.json = payload.clone();
            }
            return Ok(NodeOutput::port(port, items));
        }
        let credential_id =
            ctx.credential_ids.get("slack").cloned().ok_or_else(|| NodeError::MissingCredential("slack".into()))?;
        ctx.credential("slack").await?;
        let channel =
            ctx.settings["channel"].as_str().ok_or_else(|| NodeError::Failed("channel is required".into()))?;
        let title = ctx.settings["title"].as_str().ok_or_else(|| NodeError::Failed("title is required".into()))?;
        let message = ctx.settings["message"].as_str().unwrap_or("");
        let payload = ctx
            .settings
            .get("payload")
            .filter(|value| !value.is_null())
            .cloned()
            .unwrap_or_else(|| input.main().first().map(|item| item.json.clone()).unwrap_or(Value::Null));
        let id = ctx
            .request_approval(ApprovalSpec {
                title: title.into(),
                message: message.into(),
                payload,
                channels: vec!["slack".into()],
                target: Some(channel.into()),
                timeout_hours: None,
                credential_id: Some(credential_id),
            })
            .await?;
        Ok(NodeOutput::waiting(id))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSlackApproval)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSlackApproval);
    }

    #[tokio::test]
    async fn c02_slack_approval_passes_channel_and_credential_to_notifier() {
        let harness = NodeHarness::new(super::AppSlackApproval)
            .settings(json!({"channel":"C123","title":"Approve?","message":"Review","payload":"{{ item }}"}))
            .input_json(vec![json!({"value":1})])
            .credential("slack", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "slack".into(),
            secret: json!({"bot_token":"token"}),
            meta: json!({}),
        });
        assert!(harness.run().await.unwrap().waiting_for.is_some());
        let requests = harness.approvals.requested.lock().unwrap();
        assert_eq!(requests[0].target.as_deref(), Some("C123"));
        assert_eq!(requests[0].credential_id.as_deref(), Some("cred"));
        assert_eq!(requests[0].payload, json!({"value":1}));
    }
}
