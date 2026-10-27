//! `data.parse_document` — Parse document.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataParseDocument)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataParseDocument);
    }
}
