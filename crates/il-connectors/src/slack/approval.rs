//! `app.slack.approval` — Approval (Slack).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C02 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSlackApproval)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSlackApproval);
    }
}
