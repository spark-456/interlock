//! `data.openapi` — Use any API.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task K02 implements it.

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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataOpenapi)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataOpenapi);
    }
}
