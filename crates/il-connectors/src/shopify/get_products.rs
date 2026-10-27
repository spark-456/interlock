//! `app.shopify.get_products` — Get products (Shopify).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C03 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppShopifyGetProducts)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppShopifyGetProducts);
    }
}
