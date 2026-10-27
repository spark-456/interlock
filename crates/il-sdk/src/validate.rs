//! Minimal JSON Schema checks for node settings: `required`, property `type`, and `enum`.
//! Expressions (`{{ … }}`) are accepted for any type since they render at run time.

use il_core::NodeManifest;
use serde_json::Value;

use crate::node::ValidationIssue;

/// Default settings validation used by `NodeExecutor::validate`.
pub fn default_schema_validate(manifest: &NodeManifest, settings: &Value) -> Result<(), Vec<ValidationIssue>> {
    let mut issues = Vec::new();
    check(&manifest.settings_schema, settings, "", &mut issues);
    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

fn check(schema: &Value, value: &Value, path: &str, issues: &mut Vec<ValidationIssue>) {
    if let Value::String(s) = value
        && il_core::expr::is_expression(s)
    {
        if let Err(e) = il_core::expr::validate(s) {
            issues.push(issue(path, &e.to_string()));
        }
        return;
    }
    if let Some(ty) = schema.get("type").and_then(Value::as_str)
        && !type_matches(ty, value)
    {
        issues.push(issue(path, &format!("expected {ty}")));
        return;
    }
    if let Some(options) = schema.get("enum").and_then(Value::as_array)
        && !options.contains(value)
    {
        issues.push(issue(path, "not one of the allowed values"));
    }
    if let Value::Object(map) = value {
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for key in required.iter().filter_map(Value::as_str) {
                let missing = match map.get(key) {
                    None | Some(Value::Null) => true,
                    Some(Value::String(s)) => s.trim().is_empty(),
                    _ => false,
                };
                if missing {
                    issues.push(issue(&format!("{path}/{key}"), "is required"));
                }
            }
        }
        if let Some(props) = schema.get("properties").and_then(Value::as_object) {
            for (key, sub) in props {
                if let Some(v) = map.get(key)
                    && !v.is_null()
                {
                    check(sub, v, &format!("{path}/{key}"), issues);
                }
            }
        }
    }
    if let (Value::Array(list), Some(items)) = (value, schema.get("items")) {
        for (i, v) in list.iter().enumerate() {
            check(items, v, &format!("{path}/{i}"), issues);
        }
    }
}

fn type_matches(ty: &str, v: &Value) -> bool {
    match ty {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        _ => true,
    }
}

fn issue(path: &str, message: &str) -> ValidationIssue {
    ValidationIssue { path: if path.is_empty() { "/".into() } else { path.into() }, message: message.into() }
}

#[cfg(test)]
mod tests {
    use il_core::{Category, NodeManifest};
    use serde_json::json;

    use super::*;

    fn manifest() -> NodeManifest {
        NodeManifest::new("t.x", Category::Data, "X", "x").settings_schema(json!({
            "type": "object", "required": ["url"],
            "properties": {
                "url": {"type": "string"},
                "method": {"type": "string", "enum": ["GET", "POST"]},
                "retries": {"type": "integer"}
            }
        }))
    }

    #[test]
    fn accepts_valid_and_expressions() {
        let m = manifest();
        assert!(default_schema_validate(&m, &json!({"url": "https://x", "method": "GET"})).is_ok());
        assert!(default_schema_validate(&m, &json!({"url": "{{ item.url }}", "retries": "{{ 3 }}"})).is_ok());
    }

    #[test]
    fn reports_issues() {
        let m = manifest();
        let issues = default_schema_validate(&m, &json!({"method": "PATCH", "retries": "x"})).unwrap_err();
        let paths: Vec<_> = issues.iter().map(|i| i.path.as_str()).collect();
        assert!(paths.contains(&"/url"));
        assert!(paths.contains(&"/method"));
        assert!(paths.contains(&"/retries"));
    }
}
