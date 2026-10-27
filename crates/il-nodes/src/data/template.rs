//! `data.template` — Template.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Template node.
#[derive(Debug, Default)]
pub struct DataTemplate;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "data.template",
        Category::Data,
        "Template",
        "Renders a text, Markdown, HTML or JSON template for each item.",
    )
    .icon("template")
    .settings_schema(json!({
        "type": "object",
        "required": [
            "template"
        ],
        "properties": {
            "template": {
                "type": "string",
                "title": "Template",
                "x-widget": "longtext"
            },
            "format": {
                "type": "string",
                "title": "Format",
                "enum": [
                    "text",
                    "markdown",
                    "html",
                    "json"
                ],
                "default": "text",
                "x-widget": "select"
            },
            "outputField": {
                "type": "string",
                "title": "Output field",
                "default": "text"
            }
        }
    }))
    .example(
        "Log row",
        json!({
            "template": "{{ item.title }} posted to {{ item.platform }}",
            "format": "text"
        }),
    )
});

#[async_trait]
impl NodeExecutor for DataTemplate {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataTemplate)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataTemplate);
    }
}
