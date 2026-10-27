//! `data.parse_document` — Parse document.
//!

use il_sdk::prelude::*;

/// Parse document node.
#[derive(Debug, Default)]
pub struct DataParseDocument;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "data.parse_document",
        Category::Data,
        "Parse document",
        "Extracts text and sections from PDF, DOCX, Markdown, TXT, CSV or HTML.",
    )
    .icon("file-text")
    .status(NodeStatus::Ready)
    .settings_schema(json!({
        "type": "object",
        "required": [
            "file"
        ],
        "properties": {
            "file": {
                "type": "string",
                "title": "File",
                "default": "{{ item.files.document }}",
                "x-widget": "file"
            },
            "format": {
                "type": "string",
                "title": "Format",
                "enum": [
                    "auto",
                    "pdf",
                    "docx",
                    "md",
                    "txt",
                    "csv",
                    "html"
                ],
                "default": "auto",
                "x-widget": "select"
            }
        }
    }))
    .example(
        "Read a PDF",
        json!({
            "file": "{{ item.files.document }}",
            "format": "auto"
        }),
    )
});

#[async_trait]
impl NodeExecutor for DataParseDocument {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let file_id = ctx.settings["file"]
            .as_str()
            .or_else(|| ctx.settings["file"]["id"].as_str())
            .or_else(|| input.main().first().and_then(|item| item.files.get("document")).map(|f| f.id.as_str()))
            .filter(|id| !id.is_empty())
            .ok_or_else(|| NodeError::Failed("file must identify an uploaded document".into()))?;
        let format = ctx.settings["format"].as_str().filter(|f| *f != "auto");
        let parsed = ctx.knowledge().parse(file_id, format).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        if !item.json.is_object() {
            item.json = json!({});
        }
        item.json["document"] = parsed;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataParseDocument)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataParseDocument);
    }

    #[tokio::test]
    async fn b06_parse_document_delegates_to_knowledge_service() {
        let output = NodeHarness::new(super::DataParseDocument)
            .settings(json!({"file":"file_1","format":"txt"}))
            .input_json(vec![json!({"title":"Notes"})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"][0].json["document"]["text"], "Parsed text of file_1");
    }
}
