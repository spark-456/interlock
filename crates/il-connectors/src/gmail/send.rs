//! `app.gmail.send` — Send email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Send email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailSend;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.gmail.send",
        Category::App,
        "Send email (Gmail)",
        "Sends a new email. Recipients come from settings, not from item data, unless you map them.",
    )
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
            "to",
            "subject",
            "body"
        ],
        "properties": {
            "to": {
                "type": "string",
                "title": "To"
            },
            "subject": {
                "type": "string",
                "title": "Subject"
            },
            "body": {
                "type": "string",
                "title": "Body",
                "x-widget": "longtext"
            }
        }
    }))
    .example(
        "Weekly report",
        json!({
            "to": "team@larkspur.example",
            "subject": "Weekly promos",
            "body": "{{ item.text }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppGmailSend {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailSend)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailSend);
    }
}
