//! `app.shopify.new_product` — New product (Shopify).
//!
//! Sandbox products are polled with a persisted cursor, so each product emits once.

use il_sdk::prelude::*;
use il_sdk::{
    ctx::{TriggerCtx, TriggerHandle},
    poll::{self, Poller},
};
use std::{sync::Arc, time::Duration};

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, _ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        Ok(NodeOutput::out(input.main().to_vec()))
    }

    fn as_trigger(&self) -> Option<&dyn TriggerNode> {
        Some(self)
    }
}

#[async_trait]
impl TriggerNode for AppShopifyNewProduct {
    async fn start(&self, ctx: TriggerCtx) -> Result<TriggerHandle, NodeError> {
        if ctx.settings["store"].as_str() == Some("real") {
            return Err(NodeError::Failed("real Shopify product polling is not configured; use sandbox".into()));
        }
        let minutes = ctx.settings["pollMinutes"].as_u64().unwrap_or(5).clamp(1, 60);
        Ok(poll::spawn_polling(
            ctx,
            "app.shopify.new_product",
            Duration::from_secs(minutes * 60),
            Arc::new(SandboxPoller),
        ))
    }
}

struct SandboxPoller;

#[async_trait]
impl Poller for SandboxPoller {
    async fn poll(&self, ctx: &TriggerCtx, cursor: Option<Value>) -> Result<(Vec<Item>, Option<Value>), NodeError> {
        let products = super::sandbox_products(&ctx.services, &ctx.workspace_id).await?;
        let cursor = cursor.as_ref().and_then(Value::as_str).unwrap_or("");
        let mut newer: Vec<(String, Item)> = products
            .into_iter()
            .filter_map(|product| {
                let key = format!("{}|{}", product["createdAt"].as_str()?, product["id"].as_str()?);
                (key.as_str() > cursor).then(|| (key, Item::json(product)))
            })
            .collect();
        newer.sort_by(|a, b| a.0.cmp(&b.0));
        let next = newer.last().map(|(key, _)| json!(key));
        Ok((newer.into_iter().map(|(_, item)| item).collect(), next))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(AppShopifyNewProduct)) }

#[cfg(test)]
mod tests {
    use il_sdk::{
        ctx::TriggerCtx,
        poll::{cursor_key, poll_once},
        testkit::{FakeRunner, FakeServices},
    };
    use serde_json::json;
    use std::sync::Arc;
    use tokio_util::sync::CancellationToken;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::AppShopifyNewProduct);
    }

    #[tokio::test]
    async fn c03_sandbox_new_product_cursor_emits_once() {
        let fakes = FakeServices::default();
        let runner = Arc::new(FakeRunner::default());
        let ctx = TriggerCtx {
            flow_id: "flow".into(),
            workspace_id: "ws".into(),
            node_id: "node".into(),
            settings: json!({"store":"sandbox"}),
            services: fakes.services(),
            runner: runner.clone(),
            cancel: CancellationToken::new(),
        };
        let key = cursor_key(&ctx.flow_id, &ctx.node_id);
        assert_eq!(poll_once(&ctx, "app.shopify.new_product", &key, &super::SandboxPoller).await.unwrap(), 3);
        assert_eq!(poll_once(&ctx, "app.shopify.new_product", &key, &super::SandboxPoller).await.unwrap(), 0);
        let mut products = super::super::sandbox_products(&ctx.services, "ws").await.unwrap();
        products.push(json!({"id":"sandbox-004","title":"New","createdAt":"2026-10-25T00:00:00Z"}));
        ctx.services.store.kv_put("sandbox:products:ws", &json!(products)).await.unwrap();
        assert_eq!(poll_once(&ctx, "app.shopify.new_product", &key, &super::SandboxPoller).await.unwrap(), 1);
        assert_eq!(runner.started().len(), 4);
    }
}
