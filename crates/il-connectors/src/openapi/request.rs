//! Execution path shared by the node and Agent tool.

use std::path::PathBuf;
use std::sync::OnceLock;

use il_sdk::{SafeHttp, services::Credential};
use reqwest::Method;
use serde_json::Value;
use url::Url;

use super::spec::Operation;

/// Where recorded answers for `.example` APIs live (the templates' sandbox services).
static RECORDED: OnceLock<PathBuf> = OnceLock::new();

/// Answers calls to `<name>.example` hosts from `<dir>/<name>.mock.json`: such hosts can never
/// exist online, so the templates' APIs would otherwise always fail.
pub fn record_example_apis(dir: PathBuf) {
    let _ = RECORDED.set(dir);
}

/// The recorded answer for a call to a `.example` host. A response may carry `when` (argument values
/// it answers for, compared without case) and `otherwise` (the answer for any other values).
async fn recorded(url: &Url, operation: &Operation, args: &Value) -> Option<Result<Value, String>> {
    let name = url.host_str()?.strip_suffix(".example")?.rsplit('.').next()?.to_owned();
    let text = tokio::fs::read_to_string(RECORDED.get()?.join(format!("{name}.mock.json"))).await.ok()?;
    let mock: Value = serde_json::from_str(&text).ok()?;
    let Some(response) = mock["responses"].get(&operation.id) else {
        return Some(Err(format!("the {name} sandbox has no answer for {}", operation.id)));
    };
    let matches = response["when"].as_object().is_none_or(|when| {
        when.iter().all(|(key, want)| {
            let got = args.get(key).map(|v| v.as_str().map_or_else(|| v.to_string(), str::to_owned));
            got.is_some_and(|got| want.as_str().is_some_and(|want| got.eq_ignore_ascii_case(want)))
        })
    });
    Some(Ok(if matches { response["body"].clone() } else { response["otherwise"].clone() }))
}

/// Calls a selected operation with explicit settings parameters.
pub async fn invoke(
    http: &SafeHttp,
    operation: &Operation,
    args: &Value,
    credential: Option<&Credential>,
) -> Result<Value, String> {
    if !args.is_object() {
        return Err("parameters must be an object".into());
    }
    let mut path = operation.path.clone();
    for parameter in &operation.parameters {
        if parameter.location == "path" {
            let value = args.get(&parameter.name);
            if parameter.required && value.is_none() {
                return Err(format!("missing {}", parameter.name));
            }
            if let Some(value) = value {
                let value = scalar(value)?;
                let escaped: String = url::form_urlencoded::byte_serialize(value.as_bytes()).collect();
                path = path.replace(&format!("{{{}}}", parameter.name), &escaped);
            }
        }
    }
    if path.contains('{') {
        return Err("unresolved path parameter".into());
    }
    let mut url = Url::parse(&operation.base_url).map_err(|e| e.to_string())?;
    let prefix = url.path().trim_end_matches('/');
    let joined = format!("{prefix}/{}", path.trim_start_matches('/'));
    url.set_path(&joined);
    if let Some(query) = &operation.fixed_query {
        url.set_query(Some(query));
    }
    for parameter in &operation.parameters {
        if parameter.location == "query" {
            let value = args.get(&parameter.name);
            if parameter.required && value.is_none() {
                return Err(format!("missing {}", parameter.name));
            }
            if let Some(value) = value {
                url.query_pairs_mut().append_pair(&parameter.name, &scalar(value)?);
            }
        }
    }
    if let Some(answer) = recorded(&url, operation, args).await {
        return answer;
    }
    let method = Method::from_bytes(operation.method.as_bytes()).map_err(|e| e.to_string())?;
    let mut request = http.request(method, url.as_str()).map_err(|e| e.to_string())?;
    for parameter in &operation.parameters {
        if parameter.location == "header" {
            let value = args.get(&parameter.name);
            if parameter.required && value.is_none() {
                return Err(format!("missing {}", parameter.name));
            }
            if let Some(value) = value {
                request = request.header(&parameter.name, scalar(value)?);
            }
        }
    }
    if let Some(credential) = credential {
        let secret = &credential.secret;
        if let Some(token) = secret.get("access_token").or_else(|| secret.get("token")).and_then(Value::as_str) {
            request = request.bearer_auth(token);
        } else if let Some(key) = secret.get("api_key").and_then(Value::as_str) {
            let header = credential.meta.get("header").and_then(Value::as_str).unwrap_or("x-api-key");
            request = request.header(header, key);
        }
    }
    if let Some(body) = args.get("body").or(operation.example_body.as_ref()) {
        request = request.json(body);
    }
    let mut response =
        request.send().await.map_err(|e| e.to_string())?.error_for_status().map_err(|e| e.to_string())?;
    if response.content_length().is_some_and(|size| size > 2_000_000) {
        return Err("API response exceeds 2 MB".into());
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > 2_000_000 {
            return Err("API response exceeds 2 MB".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into())))
}

fn scalar(value: &Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        _ => Err("parameter must be a string, number or boolean".into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn q20_example_hosts_answer_from_recorded_data() {
        let dir = std::env::temp_dir().join(format!("il-recorded-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let mock =
            json!({"responses": {"find": {"when": {"email": "a@b.example"}, "body": {"id": 1}, "otherwise": []}}});
        std::fs::write(dir.join("demo.mock.json"), mock.to_string()).unwrap();
        record_example_apis(dir);
        let spec = json!({"openapi": "3.0.3", "servers": [{"url": "https://api.demo.example"}], "paths": {
            "/c": {"get": {"operationId": "find", "parameters": [{"name": "email", "in": "query"}]}},
            "/x": {"get": {"operationId": "gone"}}}});
        let ops = super::super::spec::parse(&spec.to_string()).unwrap();
        let (find, gone) = (&ops[0], &ops[1]);
        let http = SafeHttp::permissive_for_tests();
        assert_eq!(invoke(&http, find, &json!({"email": "A@B.example"}), None).await.unwrap(), json!({"id": 1}));
        assert_eq!(invoke(&http, find, &json!({"email": "someone@else.example"}), None).await.unwrap(), json!([]));
        assert!(invoke(&http, gone, &json!({}), None).await.unwrap_err().contains("no answer for gone"));
    }
}
