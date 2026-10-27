//! `logic.merge` — Merge.
//!

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        merge(&ctx.settings, &input)
    }
}

fn merge(settings: &Value, input: &NodeInput) -> Result<NodeOutput, NodeError> {
    let a = input.port("a");
    let b = input.port("b");
    let items = match settings["mode"].as_str().unwrap_or("append") {
        // The engine schedules Merge only after both incoming branches resolve.
        "append" | "waitAll" => a.iter().chain(b).cloned().collect(),
        "byKey" => {
            let key = settings["key"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| NodeError::Failed("key is required for byKey merge".into()))?;
            let mut result = a.to_vec();
            for item in b {
                let id = item.json.get(key);
                let match_index = id.and_then(|id| result.iter().position(|left| left.json.get(key) == Some(id)));
                if let Some(index) = match_index {
                    if let (Some(left), Some(right)) = (result[index].json.as_object_mut(), item.json.as_object()) {
                        left.extend(right.clone());
                    }
                    result[index].files.extend(item.files.clone());
                } else {
                    result.push(item.clone());
                }
            }
            result
        }
        _ => return Err(NodeError::Failed("unknown merge mode".into())),
    };
    Ok(NodeOutput::out(items))
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicMerge)) }

#[cfg(test)]
mod tests {
    use il_sdk::{
        prelude::{Item, NodeInput},
        testkit::NodeHarness,
    };
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicMerge);
    }

    #[tokio::test]
    async fn b06_merge_appends_joins_and_waits_for_both() {
        let mut input = NodeInput::default();
        input.ports.insert("a".into(), vec![Item::json(json!({"id":1,"a":1}))]);
        input.ports.insert("b".into(), vec![Item::json(json!({"id":1,"b":2}))]);
        let run = |mode: &str| {
            NodeHarness::new(super::LogicMerge).settings(json!({"mode":mode,"key":"id"})).ports(input.ports.clone())
        };
        assert_eq!(run("append").run().await.unwrap().ports["out"].len(), 2);
        assert_eq!(run("byKey").run().await.unwrap().ports["out"][0].json, json!({"id":1,"a":1,"b":2}));
        assert_eq!(run("waitAll").run().await.unwrap().ports["out"].len(), 2);
    }
}
