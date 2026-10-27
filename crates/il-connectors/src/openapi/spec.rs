//! OpenAPI and cURL import for the architect and API node.

use serde_json::Value;
use url::Url;

/// One callable HTTP operation.
#[derive(Debug, Clone)]
pub struct Operation {
    pub id: String,
    pub description: String,
    pub method: String,
    pub base_url: String,
    pub path: String,
    pub parameters: Vec<Parameter>,
    pub body_schema: Option<Value>,
    pub example_body: Option<Value>,
    pub fixed_query: Option<String>,
    pub risky: bool,
}

/// An operation parameter and its location.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub location: String,
    pub required: bool,
    pub schema: Value,
}

/// Imports OpenAPI 3.0/3.1 JSON or YAML, or one cURL example.
pub fn parse(source: &str) -> Result<Vec<Operation>, String> {
    if source.trim_start().starts_with("curl ") {
        return parse_curl(source).map(|operation| vec![operation]);
    }
    let document: Value = if let Ok(json) = serde_json::from_str(source) {
        json
    } else {
        serde_yaml::from_str(source).map_err(|e| format!("invalid OpenAPI document: {e}"))?
    };
    let version = document["openapi"].as_str().ok_or("missing OpenAPI version")?;
    if !version.starts_with("3.0.") && !version.starts_with("3.1.") {
        return Err(format!("unsupported OpenAPI version: {version}"));
    }
    let base_url = document["servers"][0]["url"].as_str().ok_or("missing server URL")?;
    Url::parse(base_url).map_err(|e| format!("invalid server URL: {e}"))?;
    let paths = document["paths"].as_object().ok_or("missing paths")?;
    let mut operations = Vec::new();
    for (path, methods) in paths {
        let Some(methods) = methods.as_object() else { continue };
        for (method, detail) in methods {
            if !["get", "post", "put", "patch", "delete", "head"].contains(&method.as_str()) {
                continue;
            }
            let id = detail["operationId"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| format!("{}_{}", method, path.trim_matches('/').replace(['/', '{', '}'], "_")));
            let description =
                detail["summary"].as_str().or_else(|| detail["description"].as_str()).unwrap_or(&id).to_owned();
            let mut parameters = Vec::new();
            for parent in [methods.get("parameters"), detail.get("parameters")] {
                if let Some(entries) = parent.and_then(Value::as_array) {
                    for value in entries {
                        if let (Some(name), Some(location)) = (value["name"].as_str(), value["in"].as_str()) {
                            parameters.push(Parameter {
                                name: name.into(),
                                location: location.into(),
                                required: value["required"].as_bool().unwrap_or(location == "path"),
                                schema: value["schema"].clone(),
                            });
                        }
                    }
                }
            }
            let body_schema = detail["requestBody"]["content"]["application/json"]["schema"]
                .as_object()
                .map(|_| detail["requestBody"]["content"]["application/json"]["schema"].clone());
            let risky = method == "delete"
                || ["delete", "destroy", "remove", "revoke", "refund", "cancel", "terminate"].iter().any(|word| {
                    id.to_ascii_lowercase().contains(word) || description.to_ascii_lowercase().contains(word)
                });
            operations.push(Operation {
                id,
                description,
                method: method.to_ascii_uppercase(),
                base_url: base_url.into(),
                path: path.clone(),
                parameters,
                body_schema,
                example_body: None,
                fixed_query: None,
                risky,
            });
        }
    }
    operations.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(operations)
}

/// Finds operations matching an architect hint and filters destructive calls.
pub fn select<'a>(operations: &'a [Operation], hint: &str, allow_risky: bool) -> Vec<&'a Operation> {
    let words: Vec<_> = hint.split_whitespace().map(str::to_ascii_lowercase).collect();
    operations
        .iter()
        .filter(|operation| {
            (allow_risky || !operation.risky)
                && words.iter().all(|word| {
                    operation.id.to_ascii_lowercase().contains(word)
                        || operation.description.to_ascii_lowercase().contains(word)
                        || operation.path.to_ascii_lowercase().contains(word)
                })
        })
        .collect()
}

fn parse_curl(source: &str) -> Result<Operation, String> {
    let tokens = shell_words(source)?;
    if tokens.first().map(String::as_str) != Some("curl") {
        return Err("not a cURL command".into());
    }
    let mut method = "GET".to_string();
    let mut url = None;
    let mut body = None;
    let mut index = 1;
    while index < tokens.len() {
        match tokens[index].as_str() {
            "-X" | "--request" => {
                index += 1;
                method = tokens.get(index).ok_or("missing method")?.to_ascii_uppercase();
            }
            "-d" | "--data" | "--data-raw" => {
                index += 1;
                body = Some(tokens.get(index).ok_or("missing body")?.clone());
                if method == "GET" {
                    method = "POST".into();
                }
            }
            "-H" | "--header" => {
                index += 1;
                tokens.get(index).ok_or("missing header")?;
            }
            token if token.starts_with("http://") || token.starts_with("https://") => url = Some(token.to_owned()),
            token if token.starts_with('-') => return Err(format!("unsupported cURL option: {token}")),
            _ => {}
        }
        index += 1;
    }
    let url = Url::parse(&url.ok_or("missing URL")?).map_err(|e| e.to_string())?;
    let base_url = format!("{}://{}", url.scheme(), url.host_str().ok_or("missing host")?);
    let base_url = match url.port() {
        Some(port) => format!("{base_url}:{port}"),
        None => base_url,
    };
    let path = url.path().to_string();
    let example_body = body.and_then(|value| serde_json::from_str::<Value>(&value).ok());
    let body_schema = example_body
        .as_ref()
        .map(|value| if value.is_object() { serde_json::json!({"type":"object"}) } else { serde_json::json!({}) });
    Ok(Operation {
        id: format!("{}_{}", method.to_ascii_lowercase(), path.trim_matches('/').replace('/', "_")),
        description: format!("{method} {path}"),
        risky: method == "DELETE",
        method,
        base_url,
        path,
        parameters: vec![],
        body_schema,
        example_body,
        fixed_query: url.query().map(str::to_owned),
    })
}

fn shell_words(source: &str) -> Result<Vec<String>, String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    for ch in source.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, c) if c.is_whitespace() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            _ => word.push(ch),
        }
    }
    if quote.is_some() {
        return Err("unclosed quote in cURL example".into());
    }
    if !word.is_empty() {
        words.push(word);
    }
    Ok(words)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("../../../../fixtures/apis/invoices.json");

    #[test]
    fn k02_openapi_versions_and_risk_flags() {
        let operations = parse(SAMPLE).unwrap();
        assert_eq!(operations.len(), 2);
        assert_eq!(operations.iter().find(|op| op.id == "getInvoice").unwrap().parameters[0].name, "invoiceId");
        assert!(operations.iter().find(|op| op.id == "deleteInvoice").unwrap().risky);
        assert_eq!(select(&operations, "invoice", false).len(), 1);
    }

    #[test]
    fn k02_curl_import() {
        let operations = parse(
            "curl -X POST 'https://api.example.com/items?dry_run=1' -H 'Content-Type: application/json' -d '{\"name\":\"a\"}'",
        )
        .unwrap();
        assert_eq!(operations[0].method, "POST");
        assert_eq!(operations[0].path, "/items");
        assert!(operations[0].body_schema.is_some());
        assert_eq!(operations[0].example_body, Some(serde_json::json!({"name":"a"})));
        assert_eq!(operations[0].fixed_query.as_deref(), Some("dry_run=1"));
    }

    #[test]
    fn k02_imports_openapi_30_yaml() {
        let source = "openapi: 3.0.3\nservers:\n  - url: https://api.example.com\npaths:\n  /health:\n    get:\n      operationId: health\n";
        assert_eq!(parse(source).unwrap()[0].id, "health");
    }
}
