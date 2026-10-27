//! `data.openapi` — Use any API.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task K02 implements it.

use super::{request, spec};
use il_sdk::prelude::*;

/// Use any API node.
#[derive(Debug, Default)]
pub struct DataOpenapi;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "data.openapi",
        Category::Data,
        "Use any API",
        "Calls an operation from an OpenAPI spec (URL or file); risky operations are skipped unless allowed.",
    )
    .icon("api")
    .external()
    .status(NodeStatus::Ready)
    .credential(CredentialSpec { optional: true, ..cred("api", "openapi", CredentialKind::ApiKey, "API key", &[]) })
    .settings_schema(json!({
        "type": "object",
        "required": [
            "spec",
            "operationId"
        ],
        "properties": {
            "spec": {
                "type": "string",
                "title": "OpenAPI spec",
                "description": "URL or file id."
            },
            "operationId": {
                "type": "string",
                "title": "Operation"
            },
            "params": {
                "type": "object",
                "title": "Parameters",
                "description": "Parameter → value expression."
            },
            "allowRisky": {
                "type": "boolean",
                "title": "Allow destructive operations",
                "default": false
            }
        }
    }))
    .example(
        "Get invoice",
        json!({
            "spec": "https://api.larkspur.example/openapi.json",
            "operationId": "getInvoice",
            "params": {
                "invoiceId": "{{ item.invoiceId }}"
            }
        }),
    )
});

#[async_trait]
impl NodeExecutor for DataOpenapi {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let spec_ref = ctx.settings["spec"].as_str().ok_or_else(|| NodeError::Failed("missing spec".into()))?;
        let source = if spec_ref.starts_with("http://") || spec_ref.starts_with("https://") {
            let mut response = ctx
                .http()
                .get(spec_ref)
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .send()
                .await
                .map_err(|e| NodeError::Failed(e.to_string()))?
                .error_for_status()
                .map_err(|e| NodeError::Failed(e.to_string()))?;
            if response.content_length().is_some_and(|size| size > 2_000_000) {
                return Err(NodeError::Failed("spec exceeds 2 MB".into()));
            }
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await.map_err(|e| NodeError::Failed(e.to_string()))? {
                if bytes.len() + chunk.len() > 2_000_000 {
                    return Err(NodeError::Failed("spec exceeds 2 MB".into()));
                }
                bytes.extend_from_slice(&chunk);
            }
            bytes
        } else if spec_ref.trim_start().starts_with("curl ") {
            spec_ref.as_bytes().to_vec()
        } else {
            ctx.files().get(spec_ref).await.map_err(|e| NodeError::Failed(e.to_string()))?.1.to_vec()
        };
        if source.len() > 2_000_000 {
            return Err(NodeError::Failed("spec exceeds 2 MB".into()));
        }
        let source = std::str::from_utf8(&source).map_err(|e| NodeError::Failed(e.to_string()))?;
        let operations = spec::parse(source).map_err(NodeError::Failed)?;
        let selected =
            ctx.settings["operationId"].as_str().ok_or_else(|| NodeError::Failed("missing operationId".into()))?;
        let operation = operations
            .into_iter()
            .find(|operation| operation.id == selected)
            .ok_or_else(|| NodeError::Failed(format!("operation not found: {selected}")))?;
        if operation.risky && ctx.settings["allowRisky"] != true {
            return Err(NodeError::Failed("destructive operation requires allowRisky".into()));
        }
        let credential = if ctx.credential_ids.contains_key("api") { Some(ctx.credential("api").await?) } else { None };
        let params = ctx.settings.get("params").cloned().unwrap_or_else(|| json!({}));
        let result =
            request::invoke(ctx.http(), &operation, &params, credential.as_ref()).await.map_err(NodeError::Failed)?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        item.json["response"] = result;
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataOpenapi)) }

#[cfg(test)]
mod tests {
    use il_sdk::prelude::*;
    use il_sdk::services::FileStore;
    use il_sdk::testkit::NodeHarness;
    use wiremock::{
        Mock, MockServer, ResponseTemplate,
        matchers::{method, path},
    };

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataOpenapi);
    }

    #[tokio::test]
    async fn k02_node_calls_selected_operation_from_file() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/invoices/42"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"total": 12})))
            .mount(&server)
            .await;
        let harness = NodeHarness::new(super::DataOpenapi);
        let spec = json!({"openapi":"3.0.3","servers":[{"url":server.uri()}],"paths":{
            "/invoices/{invoiceId}":{"get":{"operationId":"getInvoice","parameters":[
                {"name":"invoiceId","in":"path","required":true,"schema":{"type":"string"}}]}}
        }})
        .to_string();
        let file = harness.fakes.files.put("api.json", "application/json", spec.into()).await.unwrap();
        let output = harness
            .settings(json!({"spec":file.id,"operationId":"getInvoice","params":{"invoiceId":"42"}}))
            .input_json(vec![json!({"requestId": "abc"})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"][0].json["response"]["total"], 12);
    }

    #[tokio::test]
    async fn k02_url_spec_skips_delete_without_allow_risky() {
        let server = MockServer::start().await;
        let spec = json!({"openapi":"3.1.0","servers":[{"url":server.uri()}],"paths":{
            "/invoices/42":{"delete":{"operationId":"deleteInvoice"}}
        }});
        Mock::given(method("GET"))
            .and(path("/openapi.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(spec))
            .mount(&server)
            .await;
        let result = NodeHarness::new(super::DataOpenapi)
            .settings(json!({"spec":format!("{}/openapi.json",server.uri()),"operationId":"deleteInvoice"}))
            .run()
            .await;
        assert!(matches!(result, Err(NodeError::Failed(message)) if message.contains("allowRisky")));
        assert_eq!(server.received_requests().await.unwrap().len(), 1);
    }
}
