//! `logic.loop` — Loop over items.
//!

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
    .status(NodeStatus::Ready)
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

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let values =
            ctx.settings["field"].as_array().ok_or_else(|| NodeError::Failed("field must resolve to a list".into()))?;
        let keep_parent = ctx.settings["keepParent"].as_bool().unwrap_or(false);
        let parent = input.main().first();
        let items = values
            .iter()
            .map(|value| {
                let json = if keep_parent {
                    let mut merged = parent.and_then(|item| item.json.as_object()).cloned().unwrap_or_default();
                    if let Some(object) = value.as_object() {
                        merged.extend(object.clone());
                    } else {
                        merged.insert("value".into(), value.clone());
                    }
                    Value::Object(merged)
                } else {
                    value.clone()
                };
                Item { json, files: parent.map(|item| item.files.clone()).unwrap_or_default() }
            })
            .collect();
        Ok(NodeOutput::out(items))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(LogicLoop)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::LogicLoop);
    }

    #[tokio::test]
    async fn b06_loop_splits_list_and_keeps_parent() {
        let output = NodeHarness::new(super::LogicLoop)
            .settings(json!({"field":"{{ item.products }}","keepParent":true}))
            .input_json(vec![
                json!({"shop":"Acme","products":[{"sku":"A"},{"sku":"B"}]}),
                json!({"shop":"Beta","products":[{"sku":"C"}]}),
            ])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"].len(), 3);
        assert_eq!(output.ports["out"][1].json["shop"], "Acme");
        assert_eq!(output.ports["out"][1].json["sku"], "B");
        assert_eq!(output.ports["out"][2].json["shop"], "Beta");
    }
}
