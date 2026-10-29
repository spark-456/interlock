//! Airtable: create and list records with a personal access token.
//!
//! Owned by task C05. Add one file per node and list it here.

pub mod create_record;
pub mod list_records;

use il_sdk::prelude::*;

use crate::rest;

const API: &str = "https://api.airtable.com/v0";

fn credential() -> CredentialSpec {
    cred("airtable", "airtable", CredentialKind::ApiKey, "Airtable personal access token", &[])
}

/// The settings shared by both nodes.
fn schema(extra: Value, required: &[&str]) -> Value {
    let mut properties = json!({
        "baseId": { "type": "string", "title": "Base id", "description": "Starts with app…; it's in the base's URL." },
        "table": { "type": "string", "title": "Table", "description": "Table name or id (tbl…)." }
    });
    if let (Some(all), Some(more)) = (properties.as_object_mut(), extra.as_object()) {
        all.extend(more.clone());
    }
    let mut required: Vec<&str> = required.to_vec();
    required.splice(0..0, ["baseId", "table"]);
    json!({ "type": "object", "required": required, "properties": properties })
}

/// `…/v0/<base>/<table>` for the node's settings.
async fn table_url(ctx: &NodeCtx) -> Result<(String, String), NodeError> {
    let base_id = rest::id(ctx, "baseId")?;
    let table = rest::setting(ctx, "table")?;
    let token = rest::api_key(ctx, "airtable", "Airtable").await?;
    Ok((rest::join(&rest::base(ctx, API), &[base_id, table])?, token))
}

/// A record as one flat object: its id and fields.
fn flat(record: &Value) -> Value {
    let mut out = record["fields"].as_object().cloned().unwrap_or_default();
    out.insert("id".into(), record["id"].clone());
    Value::Object(out)
}
