//! `app.x.post` — Post (X).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C03 implements it.

use il_sdk::prelude::*;

/// Post (X) node.
#[derive(Debug, Default)]
pub struct AppXPost;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.x.post",
        Category::App,
        "Post (X)",
        "Prepares a post post. Dry-run: validates and previews the post; needs platform approval to post.",
    )
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
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
