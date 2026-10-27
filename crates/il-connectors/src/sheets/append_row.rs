//! `app.sheets.append_row` — Append row (Google Sheets).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;

/// Append row (Google Sheets) node.
#[derive(Debug, Default)]
pub struct AppSheetsAppendRow;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.sheets.append_row",
        Category::App,
        "Append row (Google Sheets)",
        "Adds a row to a sheet, mapping columns to values.",
    )
    .icon("table")
    .app("sheets")
    .external()
    .credential(cred(
        "google",
        "google",
        CredentialKind::OAuth2,
        "Google account",
        &["https://www.googleapis.com/auth/spreadsheets"],
    ))
    .settings_schema(json!({
        "type": "object",
        "required": [
            "spreadsheetId",
            "values"
        ],
        "properties": {
            "spreadsheetId": {
                "type": "string",
                "title": "Spreadsheet"
            },
            "sheet": {
                "type": "string",
                "title": "Sheet",
                "default": "Sheet1"
            },
            "values": {
                "type": "object",
                "title": "Columns",
                "description": "Column name → value expression."
            }
        }
    }))
    .example(
        "Log post",
        json!({
            "spreadsheetId": "1AbC",
            "sheet": "Log",
            "values": {
                "Product": "{{ item.title }}",
                "Status": "posted"
            }
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppSheetsAppendRow {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSheetsAppendRow)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSheetsAppendRow);
    }
}
