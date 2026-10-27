//! `logic.merge` — Merge.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Merge node.
#[derive(Debug, Default)]
pub struct LogicMerge;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new(
        "logic.merge",
        Category::Logic,
        "Merge",
        "Combines items from two branches: append, join by key, or wait for all.",
    )
    .icon("merge")
    .inputs(&["a", "b"])
    .batch()
    .settings_schema(json!({
        "type": "object",
        "properties": {
            "mode": {
                "type": "string",
                "title": "Mode",
                "enum": [
                    "append",
                    "byKey",
                    "waitAll"
                ],
                "default": "append",
                "x-widget": "select"
            },
            "key": {
                "type": "string",
                "title": "Key"
            }
        }
    }))
    .example(
        "Append both",
        json!({
            "mode": "append"
        }),
    )
});

#[async_trait]
impl NodeExecutor for LogicMerge {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicMerge)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicMerge);
    }
}
