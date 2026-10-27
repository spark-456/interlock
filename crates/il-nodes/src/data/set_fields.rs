//! `data.set_fields` — Set fields.
//!
//! T00 stub: the manifest is real (palette, settings form, architect catalog); `execute`
//! returns `NotImplemented` until task B06 implements it.

use il_sdk::prelude::*;

/// Set fields node.
#[derive(Debug, Default)]
pub struct DataSetFields;

static MANIFEST: LazyLock<NodeManifest> = LazyLock::new(|| {
    NodeManifest::new("data.set_fields", Category::Data, "Set fields", "Adds or changes fields on each item.")
        .icon("edit")
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

    async fn execute(&self, _ctx: &mut NodeCtx, _input: NodeInput) -> Result<NodeOutput, NodeError> {
        Err(NodeError::NotImplemented(MANIFEST.type_id.clone()))
    }
}

inventory::submit! { NodeFactory::new(|| Box::new(DataSetFields)) }

#[cfg(test)]
mod tests {
    #[test]
    fn manifest_is_valid() {
        il_sdk::testkit::assert_manifest_valid(&super::DataSetFields);
    }
}
