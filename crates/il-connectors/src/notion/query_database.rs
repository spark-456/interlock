//! `app.notion.query_database` — Query database (Notion).

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

/// Query database (Notion) node.
#[derive(Debug, Default)]
pub struct AppNotionQueryDatabase;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.notion.query_database",
        Category::App,
        "Query database (Notion)",
        "Reads pages from a Notion database as one item with a `pages` list of plain values; follow it with Loop to handle each page.",
    )
    .icon("search")
    .status(NodeStatus::Ready)
    .app("notion")
    .credential(super::credential())
    .settings_schema(json!({
        "type": "object",
        "required": ["databaseId"],
        "properties": {
            "databaseId": { "type": "string", "title": "Database id" },
            "filter": {
                "type": "object",
                "title": "Filter",
                "description": "Optional Notion filter, e.g. {\"property\": \"Status\", \"status\": {\"equals\": \"Open\"}}.",
                "x-widget": "json"
            },
            "limit": { "type": "integer", "title": "Most pages", "default": 25, "minimum": 1, "maximum": 100 }
        }
    }))
    .example("Open tasks", json!({ "databaseId": "1f2e3d4c5b6a79881f2e3d4c5b6a7988", "limit": 10 }))
});

#[async_trait]
impl NodeExecutor for AppNotionQueryDatabase {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        let database = rest::id(ctx, "databaseId")?;
        let mut body = json!({ "page_size": ctx.settings["limit"].as_u64().unwrap_or(25).clamp(1, 100) });
        if ctx.settings["filter"].as_object().is_some_and(|f| !f.is_empty()) {
            body["filter"] = ctx.settings["filter"].clone();
        }
        let result = super::call(ctx, Method::POST, &["databases", database, "query"], &body).await?;
        let pages: Vec<Value> = result["results"].as_array().into_iter().flatten().map(super::plain).collect();
        Ok(NodeOutput::out(vec![Item::json(json!({ "pages": pages }))]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppNotionQueryDatabase)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppNotionQueryDatabase;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&AppNotionQueryDatabase);
    }

    #[tokio::test]
    async fn c05_returns_pages_as_plain_values() {
        let server = MockServer::start().await;
        let page = json!({
            "id": "p1",
            "url": "https://notion.so/p1",
            "properties": {
                "Name": {"type": "title", "title": [{"plain_text": "Fix "}, {"plain_text": "login"}]},
                "Status": {"type": "status", "status": {"name": "Open"}},
                "Tags": {"type": "multi_select", "multi_select": [{"name": "web"}]},
                "Due": {"type": "date", "date": {"start": "2026-10-30"}},
                "Points": {"type": "number", "number": 3}
            }
        });
        Mock::given(method("POST"))
            .and(path("/databases/db1/query"))
            .and(body_partial_json(json!({"page_size": 5, "filter": {"property": "Status"}})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"results": [page]})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(AppNotionQueryDatabase)
            .settings(json!({
                "databaseId": "db1",
                "limit": 5,
                "filter": {"property": "Status", "status": {"equals": "Open"}},
                "_base": server.uri()
            }))
            .credential("notion", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "notion".into(),
            secret: json!({"api_key": "secret_x"}),
            meta: json!({}),
        });
        let out = harness.run().await.unwrap();
        assert_eq!(
            out.ports["out"][0].json["pages"][0],
            json!({
                "id": "p1", "url": "https://notion.so/p1", "Name": "Fix login", "Status": "Open",
                "Tags": ["web"], "Due": "2026-10-30", "Points": 3
            })
        );
    }

    #[tokio::test]
    async fn c05_a_rejected_key_says_where_to_fix_it() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).respond_with(ResponseTemplate::new(401)).mount(&server).await;
        let harness = NodeHarness::new(AppNotionQueryDatabase)
            .settings(json!({"databaseId": "db1", "_base": server.uri()}))
            .credential("notion", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "notion".into(),
            secret: json!({"api_key": "bad"}),
            meta: json!({}),
        });
        assert!(harness.run().await.unwrap_err().to_string().contains("Connections"));
    }
}
