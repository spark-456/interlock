//! Shopify: sandbox store (and optional real store) products + new-product trigger.
//!
//! Owned by task C03. Add one file per node and list it here.

pub mod get_products;
pub mod new_product;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use il_sdk::{NodeError, services::Services};
use serde_json::Value;

fn image(key: &str) -> Option<&'static str> {
    match key {
        "arc-lamp" => {
            Some(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/sandbox-store/images/arc-lamp.svg")))
        }
        "field-notes" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/sandbox-store/images/field-notes.svg"
        ))),
        "meridian-mug" => Some(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/sandbox-store/images/meridian-mug.svg"
        ))),
        _ => None,
    }
}

pub(super) async fn sandbox_products(services: &Services, workspace_id: &str) -> Result<Vec<Value>, NodeError> {
    let key = format!("sandbox:products:{workspace_id}");
    let value = match services.store.kv_get(&key).await.map_err(|e| NodeError::Failed(e.to_string()))? {
        Some(value) => value,
        None => serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/sandbox-store/products.json"
        )))
        .map_err(|e| NodeError::Failed(e.to_string()))?,
    };
    let mut products = value.as_array().cloned().ok_or_else(|| NodeError::Failed("invalid sandbox products".into()))?;
    for product in &mut products {
        if let Some(svg) = product["imageKey"].as_str().and_then(image) {
            product["imageUrl"] = Value::String(format!("data:image/svg+xml;base64,{}", STANDARD.encode(svg)));
        }
    }
    Ok(products)
}
