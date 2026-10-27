//! `app.x.post` — Post (X).
//!
//! Validates a post and returns a dry-run preview.

use il_sdk::prelude::*;

/// Post (X) node.
#[derive(Debug, Default)]
pub struct AppXPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.x.post", Category::App, "Post (X)", "Previews an X post; needs platform approval to post.")
        .icon("send")
        .app("x")
        .external()
        .status(NodeStatus::DryRun)
        .credential(CredentialSpec { optional: true, ..cred("x", "x", CredentialKind::OAuth2, "X account", &[]) })
        .settings_schema(json!({
            "type": "object",
            "required": [
                "text"
            ],
            "properties": {
                "text": {
                    "type": "string",
                    "title": "Post",
                    "x-widget": "longtext"
                },
                "media": {
                    "type": "string",
                    "title": "Media",
                    "x-widget": "expression"
                }
            }
        }))
        .example(
            "Preview",
            json!({
                "text": "{{ item.post }}"
            }),
        )
});

#[async_trait]
impl NodeExecutor for AppXPost {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("post text is required".into()))?;
        if text.trim().is_empty() {
            return Err(NodeError::Failed("post text is empty".into()));
        }
        let media = super::media(ctx, &input, "media", "media", false).await?;
        Ok(super::preview(input, "x", text, media, json!({})))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppXPost)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppXPost);
    }
}
