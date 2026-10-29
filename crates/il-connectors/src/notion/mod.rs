//! Notion: create pages in a database and query it, with an internal integration token.
//!
//! Owned by task C05. Add one file per node and list it here.

pub mod create_page;
pub mod query_database;

use il_sdk::prelude::*;
use reqwest::Method;

use crate::rest;

const API: &str = "https://api.notion.com/v1";
const VERSION: (&str, &str) = ("Notion-Version", "2022-06-28");

fn credential() -> CredentialSpec {
    cred("notion", "notion", CredentialKind::ApiKey, "Notion integration token", &[])
}

async fn call(ctx: &NodeCtx, method: Method, path: &[&str], body: &Value) -> Result<Value, NodeError> {
    let token = rest::api_key(ctx, "notion", "Notion").await?;
    let url = rest::join(&rest::base(ctx, API), path)?;
    rest::call(ctx, "Notion", method, &url, &token, &[VERSION], Some(body)).await
}

/// A Notion property value from a plain one: text, number or checkbox; objects pass through as Notion JSON.
fn property(value: &Value) -> Value {
    match value {
        Value::Number(_) => json!({ "number": value }),
        Value::Bool(_) => json!({ "checkbox": value }),
        Value::Object(_) => value.clone(),
        Value::Null => json!({ "rich_text": [] }),
        other => {
            let text = other.as_str().map_or_else(|| other.to_string(), str::to_string);
            json!({ "rich_text": [{ "text": { "content": text } }] })
        }
    }
}

/// A page's properties as plain values: text, numbers, dates, choices.
fn plain(page: &Value) -> Value {
    let mut out = serde_json::Map::new();
    out.insert("id".into(), page["id"].clone());
    out.insert("url".into(), page["url"].clone());
    for (name, prop) in page["properties"].as_object().into_iter().flatten() {
        let kind = prop["type"].as_str().unwrap_or_default();
        let value = &prop[kind];
        let plain = match kind {
            "title" | "rich_text" => {
                let parts = value.as_array().into_iter().flatten();
                json!(parts.filter_map(|t| t["plain_text"].as_str()).collect::<String>())
            }
            "select" | "status" => value["name"].clone(),
            "multi_select" => {
                json!(value.as_array().into_iter().flatten().map(|o| o["name"].clone()).collect::<Vec<_>>())
            }
            "date" => value["start"].clone(),
            _ => value.clone(),
        };
        out.insert(name.clone(), plain);
    }
    Value::Object(out)
}
