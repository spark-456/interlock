//! The operations an OpenAPI document offers, named and judged the same way everywhere: by the
//! OpenAPI connector when it calls them and by the architect when it hands them to an agent.

use serde_json::Value;

/// One callable operation of an OpenAPI document.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiOperation {
    pub id: String,
    pub description: String,
    /// Upper case (`GET`, `POST`…).
    pub method: String,
    pub path: String,
    /// Destructive or money-moving; agents only get these when the person allows them.
    pub risky: bool,
}

/// Parses an OpenAPI 3.0/3.1 document (JSON or YAML).
pub fn document(source: &str) -> Result<Value, String> {
    let document: Value = match serde_json::from_str(source) {
        Ok(json) => json,
        Err(_) => serde_yaml::from_str(source).map_err(|e| format!("invalid OpenAPI document: {e}"))?,
    };
    let version = document["openapi"].as_str().ok_or("missing OpenAPI version")?;
    if !version.starts_with("3.0.") && !version.starts_with("3.1.") {
        return Err(format!("unsupported OpenAPI version: {version}"));
    }
    Ok(document)
}

/// The operation id, or `<method>_<path>` when the document has none.
pub fn operation_id(method: &str, path: &str, detail: &Value) -> String {
    detail["operationId"]
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| format!("{}_{}", method, path.trim_matches('/').replace(['/', '{', '}'], "_")))
}

/// Whether an operation deletes, cancels or moves money, judged from its method, id and description.
pub fn is_risky(method: &str, id: &str, description: &str) -> bool {
    method.eq_ignore_ascii_case("delete")
        || ["delete", "destroy", "remove", "revoke", "refund", "cancel", "terminate"]
            .iter()
            .any(|word| id.to_ascii_lowercase().contains(word) || description.to_ascii_lowercase().contains(word))
}

/// Every operation in `source`, sorted by id.
pub fn operations(source: &str) -> Result<Vec<ApiOperation>, String> {
    let document = document(source)?;
    let paths = document["paths"].as_object().ok_or("missing paths")?;
    let mut out = Vec::new();
    for (path, methods) in paths {
        let Some(methods) = methods.as_object() else { continue };
        for (method, detail) in methods {
            if !["get", "post", "put", "patch", "delete", "head"].contains(&method.as_str()) {
                continue;
            }
            let id = operation_id(method, path, detail);
            let description =
                detail["summary"].as_str().or_else(|| detail["description"].as_str()).unwrap_or(&id).to_owned();
            let risky = is_risky(method, &id, &description);
            out.push(ApiOperation { id, description, method: method.to_ascii_uppercase(), path: path.clone(), risky });
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_operations_and_flags_risky_ones() {
        let spec = r#"{"openapi": "3.0.3", "servers": [{"url": "https://api.example.com"}], "paths": {
            "/users/{id}": {"get": {"operationId": "get_user", "summary": "Get a user"}, "delete": {"summary": "Remove"}},
            "/refunds": {"post": {"operationId": "create_refund"}}}}"#;
        let ops = operations(spec).unwrap();
        let ids: Vec<_> = ops.iter().map(|o| (o.id.as_str(), o.risky)).collect();
        assert_eq!(ids, [("create_refund", true), ("delete_users__id_", true), ("get_user", false)]);
        assert!(operations("openapi: 3.1.0\npaths: {}\n").unwrap().is_empty(), "YAML works");
        assert!(operations(r#"{"swagger": "2.0"}"#).is_err());
    }
}
