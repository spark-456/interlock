//! `data.template` — Template.
//!

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let rendered = &ctx.settings["template"];
        let value = if ctx.settings["format"].as_str().unwrap_or("text") == "json" {
            match rendered {
                Value::String(text) => {
                    serde_json::from_str(text).map_err(|e| NodeError::Failed(format!("invalid JSON template: {e}")))?
                }
                other => other.clone(),
            }
        } else {
            Value::String(rendered.as_str().map(str::to_owned).unwrap_or_else(|| rendered.to_string()))
        };
        let field = ctx.settings["outputField"].as_str().unwrap_or("text");
        if field.is_empty() {
            return Err(NodeError::Failed("outputField must be nonempty".into()));
        }
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json[field] = value;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataTemplate)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataTemplate);
    }

    #[tokio::test]
    async fn b06_template_renders_text_and_json() {
        let text = NodeHarness::new(super::DataTemplate)
            .settings(json!({"template":"Hello {{ item.name }}"}))
            .input_json(vec![json!({"name":"Ada"})])
            .run()
            .await
            .unwrap();
        assert_eq!(text.ports["out"][0].json["text"], "Hello Ada");
        let structured = NodeHarness::new(super::DataTemplate)
            .settings(json!({"template":"{\"n\": {{ item.n }}}","format":"json","outputField":"value"}))
            .input_json(vec![json!({"n":3})])
            .run()
            .await
            .unwrap();
        assert_eq!(structured.ports["out"][0].json["value"], json!({"n":3}));
    }
}
