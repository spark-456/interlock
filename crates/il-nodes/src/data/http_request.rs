//! `data.http` — HTTP request.
//!

use std::time::Duration;

use reqwest::{
    Method,
    header::{HeaderName, HeaderValue},
};

use il_sdk::prelude::*;

/// HTTP request node.
#[derive(Debug, Default)]
pub struct DataHttp;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "data.http",
        Category::Data,
        "HTTP request",
        "Calls any URL (public addresses only) and returns JSON, text or a file.",
    )
    .icon("globe")
    .status(NodeStatus::Ready)
    .external()
    .credential(CredentialSpec {
        optional: true,
        ..cred("auth", "http", CredentialKind::ApiKey, "API key or token", &[])
    })
    .settings_schema(json!({
        "type": "object",
        "required": [
            "url"
        ],
        "properties": {
            "method": {
                "type": "string",
                "title": "Method",
                "enum": [
                    "GET",
                    "POST",
                    "PUT",
                    "PATCH",
                    "DELETE"
                ],
                "default": "GET",
                "x-widget": "select"
            },
            "url": {
                "type": "string",
                "title": "URL",
                "x-widget": "expression"
            },
            "headers": {
                "type": "object",
                "title": "Headers"
            },
            "authType": {
                "type": "string", "title": "Authentication", "enum": ["none", "bearer", "basic", "apiKey"],
                "default": "bearer", "x-widget": "select"
            },
            "apiKeyHeader": {"type": "string", "title": "API key header", "default": "x-api-key"},
            "query": {
                "type": "object",
                "title": "Query"
            },
            "body": {
                "type": "string",
                "title": "Body",
                "x-widget": "longtext"
            },
            "responseType": {
                "type": "string",
                "title": "Response",
                "enum": [
                    "json",
                    "text",
                    "binary"
                ],
                "default": "json",
                "x-widget": "select"
            },
            "timeoutSec": {
                "type": "integer",
                "title": "Timeout (s)",
                "default": 30,
                "minimum": 1,
                "maximum": 300
            }
        }
    }))
    .example(
        "Get JSON",
        json!({
            "method": "GET",
            "url": "https://api.larkspur.example/v1/status"
        }),
    )
});

#[async_trait]
impl NodeExecutor for DataHttp {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let url = ctx.settings["url"].as_str().ok_or_else(|| NodeError::Failed("url must be text".into()))?;
        let method = ctx.settings["method"]
            .as_str()
            .unwrap_or("GET")
            .parse::<Method>()
            .map_err(|_| NodeError::Failed("invalid HTTP method".into()))?;
        let mut request = ctx.http().request(method, url).map_err(|e| NodeError::Failed(e.to_string()))?;
        if let Some(headers) = ctx.settings["headers"].as_object() {
            for (name, value) in headers {
                let name = HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| NodeError::Failed("invalid header name".into()))?;
                let value = HeaderValue::from_str(
                    value.as_str().ok_or_else(|| NodeError::Failed("header must be text".into()))?,
                )
                .map_err(|_| NodeError::Failed("invalid header value".into()))?;
                request = request.header(name, value);
            }
        }
        if let Some(query) = ctx.settings.get("query") {
            request = request.query(query);
        }
        if ctx.credential_ids.contains_key("auth") && ctx.settings["authType"].as_str() != Some("none") {
            let credential = ctx.credential("auth").await?;
            let secret = &credential.secret;
            request = match ctx.settings["authType"].as_str().unwrap_or("bearer") {
                "bearer" => request.bearer_auth(secret_value(secret, &["access_token", "token", "api_key"])?),
                "basic" => {
                    request.basic_auth(secret_value(secret, &["username"])?, Some(secret_value(secret, &["password"])?))
                }
                "apiKey" => request.header(
                    ctx.settings["apiKeyHeader"].as_str().unwrap_or("x-api-key"),
                    secret_value(secret, &["api_key", "token"])?,
                ),
                _ => return Err(NodeError::Failed("unknown auth type".into())),
            };
        }
        if let Some(body) = ctx.settings.get("body") {
            request = if let Some(text) = body.as_str() { request.body(text.to_owned()) } else { request.json(body) };
        }
        request = request.timeout(Duration::from_secs(ctx.settings["timeoutSec"].as_u64().unwrap_or(30)));
        let response = request
            .send()
            .await
            .map_err(|e| if e.is_timeout() { NodeError::Timeout } else { NodeError::Retryable(e.to_string()) })?;
        let status = response.status();
        if !status.is_success() {
            return Err(if status.is_server_error() || status.as_u16() == 429 {
                NodeError::Retryable(format!("HTTP {status}"))
            } else {
                NodeError::Failed(format!("HTTP {status}"))
            });
        }
        let mut item = input.main().first().cloned().unwrap_or_default();
        match ctx.settings["responseType"].as_str().unwrap_or("json") {
            "json" => {
                let body: Value =
                    response.json().await.map_err(|e| NodeError::Failed(format!("invalid JSON response: {e}")))?;
                item.json = json!({"status":status.as_u16(),"body":body});
            }
            "text" => {
                let body = response.text().await.map_err(|e| NodeError::Failed(e.to_string()))?;
                item.json = json!({"status":status.as_u16(),"body":body});
            }
            "binary" => {
                let mime = response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_owned();
                let bytes = response.bytes().await.map_err(|e| NodeError::Failed(e.to_string()))?;
                let file = ctx
                    .files()
                    .put("response.bin", &mime, bytes)
                    .await
                    .map_err(|e| NodeError::Failed(e.to_string()))?;
                item.files.insert("response".into(), file);
                item.json = json!({"status":status.as_u16()});
            }
            _ => return Err(NodeError::Failed("unknown response type".into())),
        }
        Ok(NodeOutput::out(vec![item]))
    }
}

fn secret_value<'a>(secret: &'a Value, names: &[&str]) -> Result<&'a str, NodeError> {
    names
        .iter()
        .find_map(|name| secret[*name].as_str())
        .ok_or_else(|| NodeError::Failed("credential is missing the configured auth value".into()))
}

inventory::submit! { NodeFactory::new(|| Box::new(DataHttp)) }

#[cfg(test)]
mod tests {
    use il_sdk::services::{Credential, FileStore};
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{header, method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataHttp);
    }

    #[tokio::test]
    async fn b06_http_gets_json_through_safe_client() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok":true})))
            .mount(&server)
            .await;
        let output = NodeHarness::new(super::DataHttp)
            .settings(json!({"url":format!("{}/status",server.uri())}))
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"][0].json["body"]["ok"], true);
    }

    #[tokio::test]
    async fn b06_http_sends_bound_bearer_credential_and_reads_text() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(header("authorization", "Bearer fake-token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ready"))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::DataHttp)
            .settings(json!({"url":server.uri(),"responseType":"text"}))
            .credential("auth", "cred_1");
        harness.fakes.store.add_credential(Credential {
            id: "cred_1".into(),
            app: "http".into(),
            secret: json!({"token":"fake-token"}),
            meta: json!({}),
        });
        let output = harness.run().await.unwrap();
        assert_eq!(output.ports["out"][0].json["body"], "ready");
    }

    #[tokio::test]
    async fn b06_http_binary_response_becomes_file() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).insert_header("content-type", "image/png").set_body_bytes(vec![1, 2, 3]),
            )
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::DataHttp).settings(json!({"url":server.uri(),"responseType":"binary"}));
        let output = harness.run().await.unwrap();
        let file = &output.ports["out"][0].files["response"];
        assert_eq!(file.mime, "image/png");
        assert_eq!(harness.fakes.files.get(&file.id).await.unwrap().1.as_ref(), &[1, 2, 3]);
    }
}
