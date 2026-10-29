//! `output.notify` — Send notification.
//!
//! Sends a notification through the configured channel adapter.

use il_sdk::prelude::*;
use il_sdk::services::{Notification, NotifyError};

/// Send notification node.
#[derive(Debug, Default)]
pub struct OutputNotify;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "output.notify",
        Category::Output,
        "Send notification",
        "Sends a short notification to a connected channel.",
    )
    .icon("bell")
    .status(NodeStatus::Ready)
    .external()
    .credential(CredentialSpec {
        optional: true,
        ..cred("slack", "slack", CredentialKind::OAuth2, "Slack workspace", &["chat:write"])
    })
    .credential(CredentialSpec {
        optional: true,
        ..cred("discord", "discord", CredentialKind::Webhook, "Discord webhook", &[])
    })
    .credential(CredentialSpec {
        optional: true,
        ..cred("telegram", "telegram", CredentialKind::BotToken, "Telegram bot", &[])
    })
    .settings_schema(json!({
        "type": "object",
        "required": [
            "message"
        ],
        "properties": {
            "channel": {
                "type": "string",
                "title": "Channel",
                "enum": [
                    "slack",
                    "discord",
                    "telegram"
                ],
                "default": "slack",
                "x-widget": "select"
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
            "target": {
                "type": "string",
                "title": "Destination"
            }
        }
    }))
    .example(
        "Tell the team",
        json!({
            "channel": "slack",
            "target": "#team",
            "title": "Promos posted",
            "message": "{{ items | length }} posts went out."
        }),
    )
});

#[async_trait]
impl NodeExecutor for OutputNotify {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let channel = ctx.settings["channel"].as_str().unwrap_or("slack");
        if !matches!(channel, "slack" | "discord" | "telegram") {
            return Err(NodeError::Failed("unsupported notification channel".into()));
        }
        let credential_id =
            ctx.credential_ids.get(channel).cloned().ok_or_else(|| NodeError::MissingCredential(channel.into()))?;
        let notification = Notification {
            workspace_id: ctx.run.workspace_id.clone(),
            channel: channel.into(),
            target: ctx.settings["target"].as_str().map(str::to_owned),
            title: ctx.settings["title"].as_str().unwrap_or_default().into(),
            body: ctx.settings["message"].as_str().unwrap_or_default().into(),
            approval_id: None,
            payload: Value::Null,
            credential_id: Some(credential_id),
        };
        let reference = ctx.services.notifier.send(&notification).await.map_err(|e| match e {
            NotifyError::UnknownChannel(_) => NodeError::Failed(e.to_string()),
            NotifyError::Send(_) => NodeError::Retryable(e.to_string()),
        })?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["notification"] = json!({"channel":channel,"reference":reference});
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(OutputNotify)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::OutputNotify);
    }

    #[tokio::test]
    async fn b08_notify_uses_configured_destination() {
        let harness = NodeHarness::new(super::OutputNotify)
            .settings(json!({"channel":"slack","target":"#ops","title":"Done","message":"Finished"}))
            .credential("slack", "cred");
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["notification"]["channel"], "slack");
        let sent = harness.fakes.notifier.sent.lock().unwrap();
        assert_eq!(sent[0].target.as_deref(), Some("#ops"));
        assert_eq!(sent[0].body, "Finished");
        assert_eq!(sent[0].credential_id.as_deref(), Some("cred"));
    }

    #[tokio::test]
    async fn n02_notify_rejects_a_channel_without_a_connection() {
        let harness = NodeHarness::new(super::OutputNotify)
            .settings(json!({"channel":"slack","target":"#ops","message":"Finished"}));
        assert!(harness.run().await.unwrap_err().to_string().contains("slack"));
        assert!(harness.fakes.notifier.sent.lock().unwrap().is_empty());
    }
}
