//! `data.set_fields` — Set fields.
//!

use il_sdk::prelude::*;

/// Set fields node.
#[derive(Debug, Default)]
pub struct DataSetFields;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("data.set_fields", Category::Data, "Set fields", "Adds or changes fields on each item.")
        .icon("edit")
        .status(NodeStatus::Ready)
        .settings_schema(json!({
            "type": "object",
            "required": [
                "fields"
            ],
            "properties": {
                "fields": {
                    "type": "array",
                    "title": "Fields",
                    "items": {
                        "type": "object",
                        "title": "Field",
                        "properties": {
                            "name": {
                                "type": "string",
                                "title": "Name"
                            },
                            "value": {
                                "type": "string",
                                "title": "Value",
                                "x-widget": "expression"
                            }
                        },
                        "required": [
                            "name"
                        ]
                    }
                },
                "keepExisting": {
                    "type": "boolean",
                    "title": "Keep other fields",
                    "default": true
                }
            }
        }))
        .example(
            "Tag source",
            json!({
                "fields": [
                    {
                        "name": "source",
                        "value": "shopify"
                    }
                ]
            }),
        )
});

#[async_trait]
impl NodeExecutor for DataSetFields {
    fn manifest(&self) -> &NodeManifest {
        &MANIFEST
    }

    async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
        let fields =
            ctx.settings["fields"].as_array().ok_or_else(|| NodeError::Failed("fields must be a list".into()))?;
        let mut item = input.main().first().cloned().unwrap_or_default();
        let mut object = if ctx.settings["keepExisting"].as_bool().unwrap_or(true) {
            item.json.as_object().cloned().unwrap_or_default()
        } else {
            serde_json::Map::new()
        };
        for field in fields {
            let name = field["name"]
                .as_str()
                .filter(|name| !name.is_empty())
                .ok_or_else(|| NodeError::Failed("field name must be nonempty".into()))?;
            object.insert(name.into(), field.get("value").cloned().unwrap_or(Value::Null));
        }
        item.json = Value::Object(object);
        Ok(NodeOutput::out(vec![item]))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataSetFields)) }

#[cfg(test)]
mod tests {
    use il_sdk::testkit::NodeHarness;
    use serde_json::json;

    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataSetFields);
    }

    #[tokio::test]
    async fn b06_set_fields_maps_values_without_losing_existing_data() {
        let output = NodeHarness::new(super::DataSetFields)
            .settings(json!({"fields":[{"name":"upper","value":"{{ item.name | upper }}"}]}))
            .input_json(vec![json!({"name":"Ada"})])
            .run()
            .await
            .unwrap();
        assert_eq!(output.ports["out"][0].json, json!({"name":"Ada","upper":"ADA"}));
    }
}
