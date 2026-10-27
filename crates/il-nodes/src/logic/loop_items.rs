//! `logic.loop` — Loop over items.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Loop over items node.
#[derive(Debug, Default)]
pub struct LogicLoop;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "logic.loop",
        Category::Logic,
        "Loop over items",
        "Splits a list field into separate items so following steps run once per item.",
    )
    .icon("repeat")
    .batch()
    .settings_schema(json!({
        "type": "object",
        "required": [
            "field"
        ],
        "properties": {
            "field": {
                "type": "string",
                "title": "List",
                "description": "The list to split, e.g. `{{ item.products }}`.",
                "x-widget": "expression"
            },
            "keepParent": {
                "type": "boolean",
                "title": "Keep parent fields",
                "default": false
            }
        }
    }))
    .example(
        "Each product",
        json!({
            "field": "{{ item.products }}"
        }),
    )
});

#[async_trait]
impl NodeExecutor for LogicLoop {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicLoop)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicLoop);
    }
}
