//! `app.shopify.get_products` — Get products (Shopify).
//!
//! Sandbox products are bundled; real stores use the GraphQL Admin API.

use il_sdk::prelude::*;

/// Get products (Shopify) node.
#[derive(Debug, Default)]
pub struct AppShopifyGetProducts;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.shopify.get_products",
        Category::App,
        "Get products (Shopify)",
        "Lists products from the sandbox store (or a real store with a token), newest first.",
    )
    .icon("shop")
    .status(NodeStatus::Ready)
    .app("shopify")
    .batch()
    .credential(CredentialSpec {
        optional: true,
        ..cred("shopify", "shopify", CredentialKind::ApiKey, "Shopify Admin API token", &[])
    })
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "store": {
                "type": "string",
                "title": "Store",
                "enum": [
                    "sandbox",
                    "real"
                ],
                "default": "sandbox",
                "x-widget": "select"
            },
            "sort": {
                "type": "string",
                "title": "Sort",
                "enum": [
                    "newest",
                    "bestSelling"
                ],
                "default": "newest",
                "x-widget": "select"
            },
            "limit": {
                "type": "integer",
                "title": "How many",
                "default": 3,
                "minimum": 1,
                "maximum": 50
            }
        }
    }))
    .example(
        "Three newest",
        json!({
            "store": "sandbox",
            "sort": "newest",
            "limit": 3
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppShopifyGetProducts {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        let limit = ctx.settings["limit"].as_u64().unwrap_or(3).clamp(1, 50) as usize;
        let sort = ctx.settings["sort"].as_str().unwrap_or("newest");
        let mut products = if ctx.settings["store"].as_str() == Some("real") {
            if sort == "bestSelling" {
                return Err(NodeError::Failed("bestSelling is available for the sandbox store only".into()));
            }
            real_products(ctx, limit).await?
        } else {
            super::sandbox_products(&ctx.services, &ctx.run.workspace_id).await?
        };
        products.sort_by(|a, b| {
            if sort == "bestSelling" {
                b["salesCount"].as_i64().unwrap_or(0).cmp(&a["salesCount"].as_i64().unwrap_or(0))
            } else {
                b["createdAt"].as_str().unwrap_or("").cmp(a["createdAt"].as_str().unwrap_or(""))
            }
        });
        Ok(NodeOutput::out(products.into_iter().take(limit).map(Item::json).collect()))
    }
}

async fn real_products(ctx: &NodeCtx, limit: usize) -> Result<Vec<Value>, NodeError> {
    let credential = ctx.credential("shopify").await?;
    let token = credential.secret["access_token"]
        .as_str()
        .ok_or_else(|| NodeError::Failed("Shopify Admin token missing".into()))?;
    let shop = credential.meta["shop"].as_str().ok_or_else(|| NodeError::Failed("Shopify domain missing".into()))?;
    let prefix =
        shop.strip_suffix(".myshopify.com").ok_or_else(|| NodeError::Failed("invalid Shopify domain".into()))?;
    if prefix.is_empty() || !prefix.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(NodeError::Failed("invalid Shopify domain".into()));
    }
    let endpoint = format!("https://{shop}/admin/api/2026-10/graphql.json");
    #[cfg(test)]
    let endpoint = ctx.settings["_endpoint"].as_str().unwrap_or(&endpoint).to_owned();
    let query = format!(
        "query {{ products(first: {limit}, sortKey: CREATED_AT, reverse: true) {{ nodes {{ id title description createdAt featuredImage {{ url altText }} variants(first: 1) {{ nodes {{ price }} }} }} }} }}"
    );
    let response = ctx
        .http()
        .post(&endpoint)
        .map_err(|e| NodeError::Failed(e.to_string()))?
        .header("X-Shopify-Access-Token", token)
        .json(&json!({"query":query}))
        .send()
        .await
        .map_err(|_| NodeError::Retryable("Shopify request failed".into()))?;
    if !response.status().is_success() {
        return Err(NodeError::Failed(format!("Shopify HTTP {}", response.status())));
    }
    let body: Value = response.json().await.map_err(|_| NodeError::Failed("invalid Shopify response".into()))?;
    if body["errors"].as_array().is_some_and(|errors| !errors.is_empty()) {
        return Err(NodeError::Failed("Shopify query rejected".into()));
    }
    let nodes = body["data"]["products"]["nodes"]
        .as_array()
        .ok_or_else(|| NodeError::Failed("Shopify products missing".into()))?;
    Ok(nodes
        .iter()
        .map(|node| {
            json!({
                "id":node["id"], "title":node["title"], "description":node["description"],
                "createdAt":node["createdAt"], "price":node["variants"]["nodes"][0]["price"],
                "imageUrl":node["featuredImage"]["url"], "imageAlt":node["featuredImage"]["altText"]
            })
        })
        .collect())
}

inventory::submit! { NodeFactory::new(|| Box::new(AppShopifyGetProducts)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::Credential;
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppShopifyGetProducts);
    }

    #[tokio::test]
    async fn c03_sandbox_products_have_images_and_sort_orders() {
        let newest = NodeHarness::new(super::AppShopifyGetProducts)
            .settings(json!({"store":"sandbox","sort":"newest","limit":3}))
            .run()
            .await
            .unwrap();
        let products = &newest.ports["out"];
        assert_eq!(products.len(), 3);
        assert_eq!(products[0].json["id"], "sandbox-003");
        assert!(products[0].json["imageUrl"].as_str().unwrap().starts_with("data:image/svg+xml;base64,"));
        let selling = NodeHarness::new(super::AppShopifyGetProducts)
            .settings(json!({"store":"sandbox","sort":"bestSelling","limit":1}))
            .run()
            .await
            .unwrap();
        assert_eq!(selling.ports["out"][0].json["id"], "sandbox-002");
    }

    #[tokio::test]
    async fn c03_real_shopify_uses_admin_graphql_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST")).and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"data":{"products":{"nodes":[{
                "id":"gid://shopify/Product/1","title":"Test lamp","description":"A lamp", "createdAt":"2026-10-01T00:00:00Z",
                "featuredImage":{"url":"https://cdn.example.test/lamp.jpg","altText":"Lamp"},
                "variants":{"nodes":[{"price":"12.00"}]}
            }]}}}))).expect(1).mount(&server).await;
        let harness = NodeHarness::new(super::AppShopifyGetProducts)
            .settings(json!({"store":"real","sort":"newest","limit":1,"_endpoint":format!("{}/graphql",server.uri())}))
            .credential("shopify", "cred");
        harness.fakes.store.add_credential(Credential {
            id: "cred".into(),
            app: "shopify".into(),
            secret: json!({"access_token":"token"}),
            meta: json!({"shop":"demo.myshopify.com"}),
        });
        let out = harness.run().await.unwrap();
        assert_eq!(out.ports["out"][0].json["title"], "Test lamp");
        let requests = server.received_requests().await.unwrap();
        assert_eq!(requests[0].headers["x-shopify-access-token"].to_str().unwrap(), "token");
    }
}
