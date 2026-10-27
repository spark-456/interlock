//! `output.respond` — Respond.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B08 implements it.

use il_sdk::prelude::*;

/// Respond node.
#[derive(Debug, Default)]
pub struct OutputRespond;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "output.respond",
        Category::Output,
        "Respond",
        "Sends the answer back to the chat, API or MCP caller (streaming when possible).",
    )
    .icon("reply")
    .outputs(&[])
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "body": {
                "type": "string",
                "title": "Answer",
                "default": "{{ item.answer }}",
                "x-widget": "expression"
            },
            "format": {
                "type": "string",
                "title": "Format",
                "enum": [
                    "text",
                    "json"
                ],
                "default": "text",
                "x-widget": "select"
            },
            "includeSources": {
                "type": "boolean",
                "title": "Include sources",
                "default": true
            }
        }
    }))
    .example(
        "Answer with sources",
        json!({
            "body": "{{ item.answer }}",
            "includeSources": true
        }),
    )
});

#[async_trait]
impl NodeExecutor for OutputRespond {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(OutputRespond)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::OutputRespond);
    }
}
