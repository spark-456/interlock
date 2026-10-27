//! `app.gmail.reply` — Reply (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Reply (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailReply;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.gmail.reply", Category::App, "Reply (Gmail)", "Replies in the original email thread.")
        .icon("mail")
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
                "threadId",
                "body"
            ],
            "properties": {
                "threadId": {
                    "type": "string",
                    "title": "Thread",
                    "default": "{{ item.threadId }}",
                    "x-widget": "expression"
                },
                "body": {
                    "type": "string",
                    "title": "Reply",
                    "x-widget": "longtext"
                },
                "cc": {
                    "type": "string",
                    "title": "Cc"
                }
            }
        }))
        .example(
            "Send draft",
            json!({
                "threadId": "{{ item.threadId }}",
                "body": "{{ item.reply }}"
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppGmailReply {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailReply)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailReply);
    }
}
