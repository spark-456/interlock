//! Flow graph (PRD §8.1).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::Json;

/// A flow: nodes, edges and groups on one canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Flow {
    pub id: String,
    pub workspace_id: String,
    pub version: u32,
    pub name: String,
    pub kind: FlowKind,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub settings: FlowSettings,
    #[serde(default)]
    pub meta: FlowMeta,
}

/// Automation (background) or AI app (chat/API/MCP).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum FlowKind {
    Automation,
    App,
}

/// One step on the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Node {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    #[serde(default = "one")]
    pub type_version: u32,
    pub name: String,
    #[serde(default)]
    pub position: Position,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default = "empty_object")]
    pub settings: Json,
    /// `{ "<credKey>": "cred_…" }`
    #[serde(default)]
    pub credentials: BTreeMap<String, String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub notes: Option<String>,
}

/// Canvas position in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// A connection from one node's output port to another node's input port.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Edge {
    pub id: String,
    pub from: PortRef,
    pub to: PortRef,
}

/// A node + port pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct PortRef {
    pub node: String,
    pub port: String,
}

/// A labelled group of nodes on the canvas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(export)]
pub struct Group {
    pub id: String,
    pub label: String,
}

/// Flow-level settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FlowSettings {
    #[serde(default = "utc")]
    pub timezone: String,
    #[serde(default)]
    pub spend_cap_usd_per_run: Option<f64>,
}

impl Default for FlowSettings {
    fn default() -> Self {
        Self { timezone: utc(), spend_cap_usd_per_run: None }
    }
}

/// Provenance of a flow.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FlowMeta {
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub goal: Option<String>,
}

fn one() -> u32 {
    1
}
fn utc() -> String {
    "UTC".into()
}
fn empty_object() -> Json {
    Json::Object(Default::default())
}

impl Flow {
    /// Finds a node by id.
    pub fn node(&self, id: &str) -> Option<&Node> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// Edges leaving `node_id`.
    pub fn outgoing<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.from.node == node_id)
    }

    /// Edges entering `node_id`.
    pub fn incoming<'a>(&'a self, node_id: &'a str) -> impl Iterator<Item = &'a Edge> + 'a {
        self.edges.iter().filter(move |e| e.to.node == node_id)
    }

    /// Nodes with no incoming edges (triggers and sources).
    pub fn roots(&self) -> impl Iterator<Item = &Node> {
        self.nodes.iter().filter(|n| self.incoming(&n.id).next().is_none())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prd_example_and_defaults() {
        let flow: Flow = serde_json::from_value(serde_json::json!({
            "id": "flw_1", "workspaceId": "ws_1", "version": 3, "name": "Weekly product promos",
            "kind": "automation",
            "nodes": [
                {"id": "n_sched", "type": "trigger.schedule", "name": "Every Monday"},
                {"id": "n_script", "type": "ai.write", "typeVersion": 1, "name": "Write video script",
                 "position": {"x": 640.0, "y": 120.0}, "group": "g_make",
                 "settings": {"instructions": "…", "input": "{{ item.title }}"}}
            ],
            "edges": [{"id": "e1", "from": {"node": "n_sched", "port": "out"}, "to": {"node": "n_script", "port": "in"}}],
            "groups": [{"id": "g_make", "label": "Make the videos"}]
        }))
        .unwrap();
        assert_eq!(flow.settings.timezone, "UTC");
        assert_eq!(flow.nodes[0].type_version, 1);
        assert_eq!(flow.roots().map(|n| n.id.as_str()).collect::<Vec<_>>(), ["n_sched"]);
        assert_eq!(flow.outgoing("n_sched").count(), 1);
        let back = serde_json::to_value(&flow).unwrap();
        assert_eq!(back["nodes"][1]["type"], "ai.write");
    }
}
