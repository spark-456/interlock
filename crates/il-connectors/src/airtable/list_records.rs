//! `app.airtable.list_records` — List records (Airtable).

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

/// List records (Airtable) node.
#[derive(Debug, Default)]
pub struct AppAirtableListRecords;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.airtable.list_records",
        Category::App,
        "List records (Airtable)",
        "Reads records from an Airtable table as one item with a `records` list; follow it with Loop to handle each record.",
    )
    .icon("table")
    .status(NodeStatus::Ready)
    .app("airtable")
    .credential(super::credential())
    .settings_schema(super::schema(
        json!({
            "view": { "type": "string", "title": "View", "description": "Optional; uses the view's filters and order." },
            "formula": {
                "type": "string",
                "title": "Filter formula",
                "description": "Optional Airtable formula, e.g. {Status} = 'Open'."
            },
            "limit": { "type": "integer", "title": "Most records", "default": 50, "minimum": 1, "maximum": 100 }
        }),
        &[],
    ))
    .example("Open orders", json!({ "baseId": "appX1y2Z3", "table": "Orders", "formula": "{Status} = 'Open'" }))
});

#[async_trait]
impl NodeExecutor for AppAirtableListRecords {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        let (url, token) = super::table_url(ctx).await?;
        let mut url = url::Url::parse(&url).map_err(|_| NodeError::Failed("invalid Airtable address".into()))?;
        {
            let mut query = url.query_pairs_mut();
            query.append_pair("maxRecords", &ctx.settings["limit"].as_u64().unwrap_or(50).clamp(1, 100).to_string());
            for (setting, param) in [("view", "view"), ("formula", "filterByFormula")] {
                if let Some(value) = ctx.settings[setting].as_str().filter(|v| !v.is_empty()) {
                    query.append_pair(param, value);
                }
            }
        }
        let result = rest::call(ctx, "Airtable", Method::GET, url.as_str(), &token, &[], None).await?;
        let records: Vec<Value> = result["records"].as_array().into_iter().flatten().map(super::flat).collect();
        Ok(NodeOutput::out(vec![Item::json(json!({ "records": records }))]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppAirtableListRecords)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppAirtableListRecords;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&AppAirtableListRecords);
    }

    #[tokio::test]
    async fn c05_lists_records_with_a_formula() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/appBase1/Orders"))
            .and(query_param("filterByFormula", "{Status} = 'Open'"))
            .and(query_param("maxRecords", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "records": [{"id": "rec1", "fields": {"Customer": "Ada", "Total": 12}}]
            })))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(AppAirtableListRecords)
            .settings(json!({
                "baseId": "appBase1", "table": "Orders", "formula": "{Status} = 'Open'", "_base": server.uri()
            }))
            .credential("airtable", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "airtable".into(),
            secret: json!({"api_key": "pat1"}),
            meta: json!({}),
        });
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["records"], json!([{"id": "rec1", "Customer": "Ada", "Total": 12}]));
    }
}
