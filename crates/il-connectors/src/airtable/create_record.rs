//! `app.airtable.create_record` — Create record (Airtable).

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

/// Create record (Airtable) node.
#[derive(Debug, Default)]
pub struct AppAirtableCreateRecord;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.airtable.create_record",
        Category::App,
        "Create record (Airtable)",
        "Adds a record to an Airtable table, mapping fields to values.",
    )
    .icon("table")
    .status(NodeStatus::Ready)
    .app("airtable")
    .external()
    .credential(super::credential())
    .settings_schema(super::schema(
        json!({
            "fields": {
                "type": "object",
                "title": "Fields",
                "description": "Field name → value. Choices and numbers are matched to the field's type."
            }
        }),
        &["fields"],
    ))
    .example(
        "Log an order",
        json!({ "baseId": "appX1y2Z3", "table": "Orders", "fields": { "Customer": "{{ item.name }}", "Total": "{{ item.total }}" } }),
    )
});

#[async_trait]
impl NodeExecutor for AppAirtableCreateRecord {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let (url, token) = super::table_url(ctx).await?;
        let fields =
            ctx.settings["fields"].as_object().ok_or_else(|| NodeError::Failed("fields is required".into()))?;
        // typecast lets Airtable turn "3" into a number and new choice names into options.
        let body = json!({ "fields": fields, "typecast": true });
        let record = rest::once(ctx, "airtable:create", async {
            let record = rest::call(ctx, "Airtable", Method::POST, &url, &token, &[], Some(&body)).await?;
            Ok(super::flat(&record))
        })
        .await?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["airtable"] = record;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppAirtableCreateRecord)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppAirtableCreateRecord;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&AppAirtableCreateRecord);
    }

    #[tokio::test]
    async fn c05_creates_a_record_once_per_item() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/appBase1/Open%20orders"))
            .and(header("authorization", "Bearer pat1"))
            .and(body_partial_json(json!({"fields": {"Customer": "Ada"}, "typecast": true})))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"id": "rec1", "fields": {"Customer": "Ada"}})),
            )
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(AppAirtableCreateRecord)
            .settings(json!({
                "baseId": "appBase1", "table": "Open orders", "fields": {"Customer": "Ada"}, "_base": server.uri()
            }))
            .credential("airtable", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "airtable".into(),
            secret: json!({"api_key": "pat1"}),
            meta: json!({}),
        });
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["airtable"], json!({"id": "rec1", "Customer": "Ada"}));
        let again = harness.run().await.unwrap();
        assert_eq!(again.ports["out"][0].json["airtable"]["id"], "rec1", "a retry returns the saved record");
    }
}
