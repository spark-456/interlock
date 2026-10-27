//! `app.linkedin.post` — Post (LinkedIn).
//!
//! Validates a post and returns a dry-run preview.

use il_sdk::prelude::*;

/// Post (LinkedIn) node.
#[derive(Debug, Default)]
pub struct AppLinkedinPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.linkedin.post",
        Category::App,
        "Post (LinkedIn)",
        "Previews a LinkedIn post; needs platform approval to post.",
    )
    .icon("send")
    .app("linkedin")
    .external()
    .status(NodeStatus::DryRun)
    .credential(CredentialSpec {
        optional: true,
        ..cred("linkedin", "linkedin", CredentialKind::OAuth2, "LinkedIn account", &[])
    })
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
            },
            "visibility": {
                "type": "string",
                "title": "Visibility",
                "enum": [
                    "public",
                    "connections"
                ],
                "default": "public",
                "x-widget": "select"
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
impl NodeExecutor for AppLinkedinPost {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let text = ctx.settings["text"].as_str().ok_or_else(|| NodeError::Failed("post text is required".into()))?;
        if text.trim().is_empty() {
            return Err(NodeError::Failed("post text is empty".into()));
        }
        let media = super::media(ctx, &input, "media", "media", false).await?;
        let visibility = ctx.settings["visibility"].as_str().unwrap_or("public");
        Ok(super::preview(input, "linkedin", text, media, json!({"visibility":visibility})))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppLinkedinPost)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppLinkedinPost);
    }
}
