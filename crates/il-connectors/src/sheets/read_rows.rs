//! `app.sheets.read_rows` — read a range into row items.

use il_sdk::prelude::*;
use url::form_urlencoded::byte_serialize;

/// Reads rows from a Google Sheet.
#[derive(Debug, Default)]
pub struct AppSheetsReadRows;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("app.sheets.read_rows", Category::App, "Read rows (Google Sheets)", "Reads sheet rows as items.")
        .icon("table")
        .app("sheets")
        .batch()
        .status(NodeStatus::Ready)
        .credential(cred(
            "google",
            "google",
            CredentialKind::OAuth2,
            "Google account",
            &["https://www.googleapis.com/auth/spreadsheets"],
        ))
        .settings_schema(json!({"type":"object","required":["spreadsheetId","range"],"properties":{
            "spreadsheetId":{"type":"string","title":"Spreadsheet"},
            "range":{"type":"string","title":"Range","default":"Sheet1!A:Z"},
            "firstRowHeaders":{"type":"boolean","title":"First row has headers","default":true}
        }}))
        .example("Read orders", json!({"spreadsheetId":"1AbC","range":"Orders!A:Z","firstRowHeaders":true}))
});

#[async_trait]
impl NodeExecutor for AppSheetsReadRows {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }
    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id = ctx.settings["spreadsheetId"]
            .as_str()
            .filter(|id| !id.is_empty() && id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')))
            .ok_or_else(|| NodeError::Failed("invalid spreadsheetId".into()))?;
        let range = ctx.settings["range"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| NodeError::Failed("missing range".into()))?;
        let token = crate::oauth::google_token(&ctx.services, &ctx.run.workspace_id, &ctx.credential_ids).await?;
        let token = token.as_str();
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://sheets.googleapis.com");
        #[cfg(not(test))]
        let base = "https://sheets.googleapis.com";
        let encoded: String = byte_serialize(range.as_bytes()).collect();
        let url = format!("{base}/v4/spreadsheets/{id}/values/{encoded}?majorDimension=ROWS");
        let response: Value = ctx
            .http()
            .get(&url)
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .json()
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let rows = response["values"].as_array().cloned().unwrap_or_default();
        let headers = ctx.settings["firstRowHeaders"].as_bool().unwrap_or(true);
        let names: Vec<String> = if headers {
            rows.first()
                .and_then(Value::as_array)
                .map(|row| {
                    row.iter()
                        .enumerate()
                        .map(|(i, cell)| {
                            cell.as_str()
                                .filter(|name| !name.is_empty())
                                .map(str::to_owned)
                                .unwrap_or_else(|| format!("column_{}", i + 1))
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut items = Vec::new();
        for row in rows.iter().skip(usize::from(headers)) {
            let cells = row.as_array().ok_or_else(|| NodeError::Failed("invalid sheet row".into()))?;
            let mut object = serde_json::Map::new();
            for i in 0..names.len().max(cells.len()) {
                let name = names.get(i).cloned().unwrap_or_else(|| format!("column_{}", i + 1));
                object.insert(name, cells.get(i).cloned().unwrap_or(Value::Null));
            }
            items.push(Item::json(Value::Object(object)));
        }
        Ok(NodeOutput::out(items))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSheetsReadRows)) }

#[cfg(test)]
mod tests {
    use il_sdk::{prelude::*, services::Credential, testkit::NodeHarness};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path_regex},
    };

    #[tokio::test]
    async fn c06_sheets_read_rows_maps_headers_and_sparse_cells() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/v4/spreadsheets/id/values/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "values":[["Order","Amount"],["A-1",12],["A-2"]]
            })))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppSheetsReadRows)
            .settings(json!({"spreadsheetId":"id","range":"Orders!A:B","firstRowHeaders":true,"_base":server.uri()}))
            .credential("google", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        let output = harness.run().await.unwrap();
        assert_eq!(output.ports["out"][0].json, json!({"Order":"A-1","Amount":12}));
        assert_eq!(output.ports["out"][1].json, json!({"Order":"A-2","Amount":null}));
    }
}
