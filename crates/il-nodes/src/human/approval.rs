//! `human.approval` — Approval.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B08 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(HumanApproval)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::HumanApproval);
    }
}
