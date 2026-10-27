//! `data.http` — HTTP request.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataHttp)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataHttp);
    }
}
