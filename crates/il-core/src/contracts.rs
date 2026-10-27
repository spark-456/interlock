//! JSON Schemas of the root contracts, written to `contracts/*.json` by `cargo xtask gen-types`.

use schemars::schema_for;

use crate::Json;

/// `(file stem, schema)` for every root contract.
pub fn json_schemas() -> Vec<(&'static str, Json)> {
    fn s<T: schemars::JsonSchema>() -> Json {
        serde_json::to_value(schema_for!(T)).unwrap_or(Json::Null)
    }
    vec![
        ("flow", s::<crate::Flow>()),
        ("node-manifest", s::<crate::NodeManifest>()),
        ("item", s::<crate::Item>()),
        ("run-event", s::<crate::RunEvent>()),
        ("build-event", s::<crate::BuildEvent>()),
        ("run", s::<crate::Run>()),
        ("approval", s::<crate::Approval>()),
        ("api-error", s::<crate::ApiError>()),
        ("build-request", s::<crate::dto::BuildRequest>()),
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn schemas_generate() {
        for (name, schema) in super::json_schemas() {
            assert!(schema.is_object(), "{name} schema should be an object");
        }
    }
}
