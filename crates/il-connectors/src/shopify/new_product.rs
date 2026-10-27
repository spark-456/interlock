//! `app.shopify.new_product` — New product (Shopify).
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task C03 implements it.

use il_sdk::prelude::*;

/// New product (Shopify) node.
#[derive(Debug, Default)]
pub struct AppShopifyNewProduct;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "app.shopify.new_product",
        Category::Trigger,
        "New product (Shopify)",
        "Starts the flow when a product is added to the store.",
    )
    .icon("shop")
    .app("shopify")
    .trigger()
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
            "pollMinutes": {
                "type": "integer",
                "title": "Check every (min)",
                "default": 5,
                "minimum": 1,
                "maximum": 60
            }
        }
    }))
    .example(
        "Sandbox store",
        json!({
            "store": "sandbox"
        }),
    )
});

#[async_trait]
impl NodeExecutor for AppShopifyNewProduct {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppShopifyNewProduct)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppShopifyNewProduct);
    }
}
