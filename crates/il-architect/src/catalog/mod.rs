//! Compact catalog for prompts, built from registered manifests (never hardcoded types).

use std::collections::BTreeMap;
use std::fmt::Write;

use il_core::{NodeManifest, NodeStatus, PortSpec};
use serde_json::Value;

/// The node types the architect may use.
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    manifests: BTreeMap<String, NodeManifest>,
}

impl Catalog {
    /// A catalog of these manifests.
    pub fn new(manifests: impl IntoIterator<Item = NodeManifest>) -> Self {
        Self { manifests: manifests.into_iter().map(|m| (m.type_id.clone(), m)).collect() }
    }

    /// Every node type registered in this binary.
    pub fn registered() -> Self {
        Self::new(il_sdk::registry::catalog())
    }

    /// A manifest by type id.
    pub fn get(&self, type_id: &str) -> Option<&NodeManifest> {
        self.manifests.get(type_id)
    }

    /// One short entry per node type: ports, settings (required marked `*`), connections and an example.
    pub fn prompt(&self) -> String {
        let mut out = String::new();
        for m in self.manifests.values() {
            let trigger = if m.trigger { " (trigger)" } else { "" };
            let _ = write!(out, "- {}{trigger} \"{}\": {}", m.type_id, m.title, m.description);
            if m.status == NodeStatus::DryRun {
                out.push_str(" Runs as a dry-run.");
            }
            let _ = write!(out, "\n  ports: {} -> {}", ports(&m.inputs), ports(&m.outputs));
            if let Some(key) = &m.dynamic_outputs {
                let _ = write!(out, " + one per `{key}` entry name");
            }
            let settings = settings(&m.settings_schema);
            if !settings.is_empty() {
                let _ = write!(out, "\n  settings: {settings}");
            }
            if !m.credentials.is_empty() {
                let apps: Vec<_> = m.credentials.iter().map(|c| c.app.as_str()).collect();
                let _ = write!(out, "\n  connects to: {}", apps.join(", "));
            }
            if let Some(ex) = m.examples.first() {
                let _ = write!(out, "\n  example: {}", ex.settings);
            }
            out.push('\n');
        }
        out
    }
}

fn ports(p: &[PortSpec]) -> String {
    if p.is_empty() { "none".into() } else { p.iter().map(|p| p.name.as_str()).collect::<Vec<_>>().join("|") }
}

/// `key* (shape) = default` for each property.
fn settings(schema: &Value) -> String {
    let required: Vec<&str> =
        schema.get("required").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str).collect();
    let Some(props) = schema.get("properties").and_then(Value::as_object) else { return String::new() };
    props
        .iter()
        .map(|(key, s)| {
            let star = if required.contains(&key.as_str()) { "*" } else { "" };
            let default = s.get("default").map(|d| format!(" = {d}")).unwrap_or_default();
            format!("{key}{star} ({}){default}", shape(s))
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Enum options, the widget hint, or the JSON type (with list item and object shapes).
fn shape(s: &Value) -> String {
    if let Some(options) = s.get("enum").and_then(Value::as_array) {
        return options
            .iter()
            .map(|o| o.as_str().map_or_else(|| o.to_string(), String::from))
            .collect::<Vec<_>>()
            .join("|");
    }
    if let Some(w) = s.get("x-widget").and_then(Value::as_str).filter(|w| *w != "select") {
        return w.into();
    }
    match s.get("type").and_then(Value::as_str) {
        Some("array") => format!("list of {}", s.get("items").map_or_else(|| "values".into(), shape)),
        Some("object") => match s.get("properties").and_then(Value::as_object) {
            Some(p) if !p.is_empty() => format!("{{{}}}", p.keys().cloned().collect::<Vec<_>>().join(", ")),
            _ => "object".into(),
        },
        Some(t) => t.into(),
        None => "any".into(),
    }
}

#[cfg(test)]
mod tests {
    use il_core::{Category, NodeManifest};
    use serde_json::json;

    use super::*;

    #[test]
    fn renders_compact_entries() {
        let m = NodeManifest::new("ai.write", Category::Ai, "Write", "Writes text.").settings_schema(json!({
            "type": "object", "required": ["instructions"],
            "properties": {
                "instructions": {"type": "string", "x-widget": "longtext"},
                "tone": {"type": "string", "enum": ["neutral", "upbeat"], "default": "neutral"},
                "labels": {"type": "array", "items": {"type": "object", "properties": {"name": {"type": "string"}}}}
            }
        }));
        let text = Catalog::new([m]).prompt();
        assert!(text.starts_with("- ai.write \"Write\": Writes text.\n  ports: in -> out\n"), "{text}");
        assert!(text.contains("instructions* (longtext)"));
        assert!(text.contains("tone (neutral|upbeat) = \"neutral\""));
        assert!(text.contains("labels (list of {name})"));
    }
}
