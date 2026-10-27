//! `app.gmail.new_email` — New email (Gmail).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// New email (Gmail) node.
#[derive(Debug, Default)]
pub struct AppGmailNewEmail;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.gmail.new_email",
        Category::Trigger,
        "New email (Gmail)",
        "Starts the flow for each new email matching a label or search.",
    )
    .icon("mail")
    .app("gmail")
    .trigger()
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/gmail.modify"],
    ))
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "label": {
                "type": "string",
                "title": "Label",
                "default": "INBOX"
            },
            "query": {
                "type": "string",
                "title": "Search",
                "description": "Gmail search, e.g. `to:support@larkspur.example`."
            },
            "pollMinutes": {
                "type": "integer",
                "title": "Check every (min)",
                "default": 1,
                "minimum": 1,
                "maximum": 60
            }
        }
    }))
    .example(
        "Support inbox",
        json!({
            "label": "INBOX",
            "query": "to:support@larkspur.example"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppGmailNewEmail {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppGmailNewEmail)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppGmailNewEmail);
    }
}
