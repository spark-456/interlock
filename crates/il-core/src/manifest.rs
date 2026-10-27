//! Node manifests (PRD §8.3). The manifest is the interface: the palette, settings form,
//! architect catalog and `docs/NODES.md` are all generated from it.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Json;

/// Everything Interlock knows about a node type without running it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct NodeManifest {
    /// Dotted type id, e.g. `ai.write`, `app.gmail.reply`.
    pub type_id: String,
    pub version: u32,
    pub category: Category,
    pub title: String,
    /// One or two sentences; also shown to the architect.
    pub description: String,
    /// Icon key (web icon set).
    pub icon: String,
    /// Logo slot key for app nodes (`gmail`, `slack`…).
    pub app: Option<String>,
    /// True for nodes that start runs (they have no inputs).
    pub trigger: bool,
    pub inputs: Vec<PortSpec>,
    pub outputs: Vec<PortSpec>,
    /// Settings key whose entries (each with a `name`) become extra output ports, e.g. Switch `cases`.
    pub dynamic_outputs: Option<String>,
    pub exec_mode: ExecMode,
    /// JSON Schema for settings. `x-widget` hints: expression, credential, file, longtext, select, model-role.
    pub settings_schema: Json,
    pub credentials: Vec<CredentialSpec>,
    pub side_effect: SideEffect,
    pub status: NodeStatus,
    pub cost_hint: CostHint,
    pub examples: Vec<NodeExample>,
}

/// Palette category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum Category {
    Trigger,
    Ai,
    Knowledge,
    Media,
    Logic,
    Data,
    App,
    Human,
    Output,
}

/// An input or output port.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct PortSpec {
    pub name: String,
    pub label: Option<String>,
}

impl PortSpec {
    /// A port with no display label.
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), label: None }
    }
}

/// Whether the engine calls the node once per item or once with all items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum ExecMode {
    PerItem,
    Batch,
}

/// A credential the node needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CredentialSpec {
    /// Key used in `Node.credentials` and `ctx.credential(key)`.
    pub key: String,
    /// Connection app (`google`, `slack`, `bluesky`, `openapi`…).
    pub app: String,
    pub kind: CredentialKind,
    pub label: String,
    pub scopes: Vec<String>,
    pub optional: bool,
}

/// How a credential is obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CredentialKind {
    OAuth2,
    ApiKey,
    BotToken,
    AppPassword,
    Webhook,
}

/// Whether running the node changes anything outside Interlock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SideEffect {
    None,
    External,
}

/// Implementation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum NodeStatus {
    Ready,
    /// Validates and previews but never posts (needs platform approval).
    DryRun,
    Experimental,
    /// Manifest only (T00); the executor returns `NotImplemented`. Usable in templates and the architect.
    Stub,
}

/// Rough cost per run of the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum CostHint {
    Free,
    Low,
    Medium,
    High,
}

/// A short example the architect sees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct NodeExample {
    pub title: String,
    pub settings: Json,
}

impl NodeManifest {
    /// Starts a manifest with defaults: one `in` and one `out` port, per-item, no side effect, stub.
    pub fn new(type_id: &str, category: Category, title: &str, description: &str) -> Self {
        Self {
            type_id: type_id.into(),
            version: 1,
            category,
            title: title.into(),
            description: description.into(),
            icon: "box".into(),
            app: None,
            trigger: false,
            inputs: vec![PortSpec::new("in")],
            outputs: vec![PortSpec::new("out")],
            dynamic_outputs: None,
            exec_mode: ExecMode::PerItem,
            settings_schema: serde_json::json!({"type": "object", "properties": {}}),
            credentials: vec![],
            side_effect: SideEffect::None,
            status: NodeStatus::Stub,
            cost_hint: CostHint::Free,
            examples: vec![],
        }
    }

    /// Sets the icon key.
    pub fn icon(mut self, icon: &str) -> Self {
        self.icon = icon.into();
        self
    }
    /// Sets the app logo slot.
    pub fn app(mut self, app: &str) -> Self {
        self.app = Some(app.into());
        self
    }
    /// Marks the node as a trigger (no inputs).
    pub fn trigger(mut self) -> Self {
        self.trigger = true;
        self.inputs.clear();
        self
    }
    /// Replaces input ports.
    pub fn inputs(mut self, names: &[&str]) -> Self {
        self.inputs = names.iter().map(|n| PortSpec::new(n)).collect();
        self
    }
    /// Replaces output ports.
    pub fn outputs(mut self, names: &[&str]) -> Self {
        self.outputs = names.iter().map(|n| PortSpec::new(n)).collect();
        self
    }
    /// Extra outputs derived from a settings array.
    pub fn dynamic_outputs(mut self, settings_key: &str) -> Self {
        self.dynamic_outputs = Some(settings_key.into());
        self
    }
    /// Batch execution.
    pub fn batch(mut self) -> Self {
        self.exec_mode = ExecMode::Batch;
        self
    }
    /// Settings JSON Schema.
    pub fn settings_schema(mut self, schema: Json) -> Self {
        self.settings_schema = schema;
        self
    }
    /// Adds a credential requirement.
    pub fn credential(mut self, spec: CredentialSpec) -> Self {
        self.credentials.push(spec);
        self
    }
    /// Marks the node as changing things outside Interlock.
    pub fn external(mut self) -> Self {
        self.side_effect = SideEffect::External;
        self
    }
    /// Sets the status.
    pub fn status(mut self, status: NodeStatus) -> Self {
        self.status = status;
        self
    }
    /// Sets the cost hint.
    pub fn cost(mut self, cost: CostHint) -> Self {
        self.cost_hint = cost;
        self
    }
    /// Adds an example.
    pub fn example(mut self, title: &str, settings: Json) -> Self {
        self.examples.push(NodeExample { title: title.into(), settings });
        self
    }

    /// Output port names for a node with these settings (static + dynamic).
    pub fn output_ports(&self, settings: &Json) -> Vec<String> {
        let mut ports: Vec<String> = self.outputs.iter().map(|p| p.name.clone()).collect();
        if let Some(key) = &self.dynamic_outputs
            && let Some(list) = settings.get(key).and_then(Json::as_array)
        {
            for entry in list {
                let name = entry.get("name").and_then(Json::as_str).or_else(|| entry.as_str());
                if let Some(name) = name {
                    ports.push(name.to_string());
                }
            }
        }
        ports
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dynamic_outputs_follow_settings() {
        let m = NodeManifest::new("logic.switch", Category::Logic, "Switch", "Routes items.")
            .outputs(&["default"])
            .dynamic_outputs("cases");
        let ports = m.output_ports(&serde_json::json!({"cases": [{"name": "billing"}, "spam"]}));
        assert_eq!(ports, ["default", "billing", "spam"]);
    }

    #[test]
    fn trigger_has_no_inputs() {
        let m = NodeManifest::new("trigger.manual", Category::Trigger, "Manual", "Run by hand.").trigger();
        assert!(m.inputs.is_empty());
        assert!(m.trigger);
    }
}
