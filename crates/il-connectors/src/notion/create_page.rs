//! `app.notion.create_page` — Create page (Notion).

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

/// Create page (Notion) node.
#[derive(Debug, Default)]
pub struct AppNotionCreatePage;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.notion.create_page",
        Category::App,
        "Create page (Notion)",
        "Adds a page to a Notion database, with a title, properties and body text.",
    )
    .icon("file-plus")
    .status(NodeStatus::Ready)
    .app("notion")
    .external()
    .credential(super::credential())
    .settings_schema(json!({
        "type": "object",
        "required": ["databaseId", "title"],
        "properties": {
            "databaseId": {
                "type": "string",
                "title": "Database id",
                "description": "From the database link: notion.so/…/<database id>?v=…. Share the database with your integration."
            },
            "title": { "type": "string", "title": "Title", "x-widget": "expression" },
            "titleProperty": { "type": "string", "title": "Title column", "default": "Name" },
            "properties": {
                "type": "object",
                "title": "Other columns",
                "description": "Column name → value. Text, numbers and true/false map to Notion text, number and checkbox."
            },
            "content": { "type": "string", "title": "Page text", "x-widget": "longtext" }
        }
    }))
    .example(
        "Log a lead",
        json!({
            "databaseId": "1f2e3d4c5b6a79881f2e3d4c5b6a7988",
            "title": "{{ item.name }}",
            "properties": { "Email": "{{ item.email }}", "Score": "{{ item.score }}" },
            "content": "{{ item.summary }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppNotionCreatePage {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let database = rest::id(ctx, "databaseId")?;
        let title = rest::setting(ctx, "title")?;
        let title_property = ctx.settings["titleProperty"].as_str().filter(|s| !s.is_empty()).unwrap_or("Name");
        let mut properties: serde_json::Map<String, Value> = ctx.settings["properties"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(name, value)| (name.clone(), super::property(value)))
            .collect();
        properties.insert(title_property.into(), json!({ "title": [{ "text": { "content": title } }] }));
        // Notion caps one text block at 2000 characters.
        let children: Vec<Value> = ctx.settings["content"]
            .as_str()
            .unwrap_or_default()
            .split("\n\n")
            .filter(|p| !p.trim().is_empty())
            .map(|p| {
                let text: String = p.chars().take(2000).collect();
                json!({ "type": "paragraph", "paragraph": { "rich_text": [{ "text": { "content": text } }] } })
            })
            .collect();
        let body = json!({ "parent": { "database_id": database }, "properties": properties, "children": children });
        let page = rest::once(ctx, "notion:page", async {
            let page = super::call(ctx, Method::POST, &["pages"], &body).await?;
            Ok(json!({ "id": page["id"], "url": page["url"] }))
        })
        .await?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["notion"] = page;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppNotionCreatePage)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::matchers::{body_partial_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::AppNotionCreatePage;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&AppNotionCreatePage);
    }

    #[tokio::test]
    async fn c05_creates_a_page_with_properties_and_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/pages"))
            .and(header("authorization", "Bearer secret_x"))
            .and(header("notion-version", "2022-06-28"))
            .and(body_partial_json(json!({
                "parent": {"database_id": "db1"},
                "properties": {
                    "Name": {"title": [{"text": {"content": "Ada"}}]},
                    "Score": {"number": 9},
                    "Email": {"rich_text": [{"text": {"content": "ada@example.com"}}]}
                },
                "children": [{"type": "paragraph"}, {"type": "paragraph"}]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"id": "p1", "url": "https://notion.so/p1"})))
            .expect(1)
            .mount(&server)
            .await;
        let harness = NodeHarness::new(AppNotionCreatePage)
            .settings(json!({
                "databaseId": "db1",
                "title": "Ada",
                "properties": {"Score": 9, "Email": "ada@example.com"},
                "content": "First paragraph.\n\nSecond.",
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
        assert_eq!(out.ports["out"][0].json["notion"]["url"], "https://notion.so/p1");
    }

    #[tokio::test]
    async fn c05_rejects_ids_that_could_change_the_path() {
        let harness = NodeHarness::new(AppNotionCreatePage)
            .settings(json!({"databaseId": "../users", "title": "x"}))
            .credential("notion", "cred");
        assert!(harness.run().await.unwrap_err().to_string().contains("databaseId"));
    }
}
