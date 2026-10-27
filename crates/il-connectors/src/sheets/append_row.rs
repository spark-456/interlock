//! `app.sheets.append_row` — Append row (Google Sheets).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C01 implements it.

use il_sdk::prelude::*;
use url::form_urlencoded::byte_serialize;

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let id =
            ctx.settings["spreadsheetId"].as_str().ok_or_else(|| NodeError::Failed("missing spreadsheetId".into()))?;
        if id.is_empty() || !id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
            return Err(NodeError::Failed("invalid spreadsheetId".into()));
        }
        let sheet = ctx.settings["sheet"].as_str().unwrap_or("Sheet1");
        let values =
            ctx.settings["values"].as_object().ok_or_else(|| NodeError::Failed("values must be an object".into()))?;
        let key = format!("sheets:append:{}", ctx.dedupe_key());
        if let Some(saved) = ctx.services.store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
            let mut item = input.main().first().cloned().unwrap_or_default();
            item.json["sheets"] = saved;
            return Ok(NodeOutput::out(vec![item]));
        }
        let credential = ctx.credential("google").await?;
        let token = credential.secret["access_token"]
            .as_str()
            .ok_or_else(|| NodeError::Failed("missing Google access token".into()))?;
        #[cfg(test)]
        let base = ctx.settings["_base"].as_str().unwrap_or("https://sheets.googleapis.com");
        #[cfg(not(test))]
        let base = "https://sheets.googleapis.com";
        let range: String = byte_serialize(format!("{sheet}!1:1").as_bytes()).collect();
        let root = format!("{base}/v4/spreadsheets/{id}/values/{range}");
        let header_response = ctx
            .http()
            .get(&root)
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        let header: Value = header_response.json().await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let headings: Vec<String> = header["values"][0]
            .as_array()
            .map(|row| row.iter().filter_map(Value::as_str).map(str::to_owned).collect())
            .unwrap_or_else(|| values.keys().cloned().collect());
        if values.keys().any(|name| !headings.contains(name)) {
            return Err(NodeError::Failed("a configured column is absent from the sheet header".into()));
        }
        let row: Vec<Value> = headings
            .iter()
            .map(|heading| values.get(heading).cloned().unwrap_or(Value::String(String::new())))
            .collect();
        let rows = if header["values"][0].as_array().is_some() {
            vec![row]
        } else {
            vec![headings.iter().map(|heading| json!(heading)).collect(), row]
        };
        let append = format!(
            "{base}/v4/spreadsheets/{id}/values/{range}:append?valueInputOption=RAW&insertDataOption=INSERT_ROWS"
        );
        let result: Value = ctx
            .http()
            .post(&append)
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .bearer_auth(token)
            .json(&json!({"majorDimension":"ROWS","values":rows}))
            .send()
            .await
            .map_err(|e| NodeError::Retryable(e.to_string()))?
            .error_for_status()
            .map_err(|e| NodeError::Failed(e.to_string()))?
            .json()
            .await
            .map_err(|e| NodeError::Failed(e.to_string()))?;
        ctx.services.store.kv_put(&key, &result).await.map_err(|e| NodeError::Failed(e.to_string()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["sheets"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppSheetsAppendRow)) }

#[cfg(test)]
mod tests {
    use il_sdk::{prelude::*, services::Credential, testkit::NodeHarness};
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path_regex},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppSheetsAppendRow);
    }

    #[tokio::test]
    async fn c01_sheets_append_maps_values_to_header_order() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/v4/spreadsheets/id/values/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"values":[["Status","Product"]]})))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path_regex("/v4/spreadsheets/id/values/.*"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"updates":{"updatedRows":1}})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::AppSheetsAppendRow)
            .settings(json!({"spreadsheetId":"id","sheet":"Log","values":{"Product":"Hat","Status":"posted"},"_base":server.uri()}))
            .credential("google","cred_1");
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "google".into(),
            secret: json!({"access_token":"test"}),
            meta: json!({}),
        });
        assert_eq!(harness.run().await.unwrap().ports["out"][0].json["sheets"]["updates"]["updatedRows"], 1);
        let requests = server.received_requests().await.unwrap();
        let append: Value = serde_json::from_slice(&requests[1].body).unwrap();
        assert_eq!(append["values"][0], json!(["posted", "Hat"]));
    }
}
