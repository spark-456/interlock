//! `app.gmail.label` — Label email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Label email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailLabel;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.gmail.label", Category::App, "Label email (Gmail)", "Adds or removes labels on an email.")
        .icon("tag")
        .app("gmail")
        .external()
        .credential(cred(
            "google",
            "google",
            CredentialKind::OAuth2,
            "Google account",
            &["https://www.googleapis.com/auth/gmail.modify"],
        ))
        .settings_schema(json!({
            "type": "object",
            "required": [
                "messageId"
            ],
            "properties": {
                "messageId": {
                    "type": "string",
                    "title": "Message",
                    "default": "{{ item.messageId }}",
                    "x-widget": "expression"
                },
                "addLabels": {
                    "type": "array",
                    "title": "Add labels",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                },
                "removeLabels": {
                    "type": "array",
                    "title": "Remove labels",
                    "items": {
                        "type": "string"
                    },
                    "default": []
                }
            }
        }))
        .example(
            "Mark spam",
            json!({
                "messageId": "{{ item.messageId }}",
                "addLabels": [
                    "SPAM"
                ]
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppGmailLabel {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailLabel)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailLabel);
    }
}
