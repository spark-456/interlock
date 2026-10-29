//! Flow validation: known types, settings schemas, edges and ports, cycles, and expressions that
//! reference real, earlier steps. The issues feed the repair loop, so messages speak to the model.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use il_core::{Flow, Node};
use serde_json::Value;

use crate::catalog::Catalog;

/// One problem, located by node or edge id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub path: String,
    pub message: String,
}

impl Issue {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self { path: path.into(), message: message.into() }
    }
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Every problem in `flow` (empty when it is valid).
pub fn validate(flow: &Flow, catalog: &Catalog) -> Vec<Issue> {
    let mut issues = Vec::new();
    let mut ids = BTreeSet::new();
    let groups: BTreeSet<&str> = flow.groups.iter().map(|g| g.id.as_str()).collect();
    let available_tools: BTreeSet<&str> = flow
        .nodes
        .iter()
        .filter(|n| !n.disabled)
        .flat_map(|n| match n.node_type.as_str() {
            "data.openapi" => n.settings.get("operationId").and_then(Value::as_str).into_iter().collect::<Vec<_>>(),
            "ai.agent" => n.settings["apiTools"]["operations"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect(),
            _ => Vec::new(),
        })
        .collect();
    for n in &flow.nodes {
        let at = format!("nodes.{}", n.id);
        if !ids.insert(n.id.as_str()) {
            issues.push(Issue::new(&at, "duplicate node id"));
        }
        if let Some(g) = n.group.as_deref().filter(|g| !groups.contains(g)) {
            issues.push(Issue::new(&at, format!("group `{g}` is not in `groups`")));
        }
        let Some(m) = catalog.get(&n.node_type) else {
            issues.push(Issue::new(&at, format!("unknown node type `{}`; use a type from the catalog", n.node_type)));
            continue;
        };
        if let Err(errs) = il_sdk::validate::validate_json(&m.settings_schema, &n.settings) {
            issues.extend(errs.into_iter().map(|e| Issue::new(format!("{at}.settings{}", e.path), e.message)));
        }
        if n.node_type == "ai.agent" {
            for (index, tool) in n.settings["tools"].as_array().into_iter().flatten().enumerate() {
                if let Some(name) = tool.as_str().filter(|name| !available_tools.contains(name)) {
                    issues.push(Issue::new(
                        format!("{at}.settings.tools[{index}]"),
                        format!("tool `{name}` is unavailable in this flow"),
                    ));
                }
            }
        }
        if !m.inputs.is_empty() && !m.trigger && flow.incoming(&n.id).next().is_none() {
            issues.push(Issue::new(&at, "nothing connects into this step, so it never runs"));
        }
    }
    if !flow.nodes.iter().any(|n| catalog.get(&n.node_type).is_some_and(|m| m.trigger)) {
        issues.push(Issue::new("nodes", "the flow needs a trigger step to start it"));
    }
    for e in &flow.edges {
        let at = format!("edges.{}", e.id);
        let (from, to) = (flow.node(&e.from.node), flow.node(&e.to.node));
        if from.is_none() || to.is_none() {
            issues.push(Issue::new(&at, format!("connects unknown step `{}` -> `{}`", e.from.node, e.to.node)));
            continue;
        }
        if let Some(from) = from
            && let Some(outs) = output_ports(from, catalog)
            && !outs.contains(&e.from.port)
        {
            issues.push(Issue::new(
                &at,
                format!("`{}` has no output `{}`; outputs: {}", from.id, e.from.port, outs.join(", ")),
            ));
        }
        if let Some(m) = to.and_then(|n| catalog.get(&n.node_type))
            && !m.inputs.iter().any(|p| p.name == e.to.port)
        {
            issues.push(Issue::new(&at, format!("`{}` has no input `{}`", e.to.node, e.to.port)));
        }
    }
    let before = ancestors(flow);
    if let Some(n) = flow.nodes.iter().find(|n| before[n.id.as_str()].contains(n.id.as_str())) {
        issues.push(Issue::new(
            format!("nodes.{}", n.id),
            "the flow loops back to this step; flows must not have cycles",
        ));
    }
    for n in &flow.nodes {
        for r in references(&n.settings) {
            if flow.node(&r).is_none() {
                issues.push(Issue::new(format!("nodes.{}.settings", n.id), format!("references unknown step `{r}`")));
            } else if !before[n.id.as_str()].contains(r.as_str()) {
                issues.push(Issue::new(
                    format!("nodes.{}.settings", n.id),
                    format!("references `{r}`, which does not run before it"),
                ));
            }
        }
    }
    issues
}

/// Output port names, including ports named by `dynamic_outputs` entries. `None` for unknown types.
pub fn output_ports(node: &Node, catalog: &Catalog) -> Option<Vec<String>> {
    let m = catalog.get(&node.node_type)?;
    let mut ports: Vec<String> = m.outputs.iter().map(|p| p.name.clone()).collect();
    if let Some(entries) = m.dynamic_outputs.as_deref().and_then(|k| node.settings.get(k)).and_then(Value::as_array) {
        ports.extend(entries.iter().filter_map(|e| e.get("name")?.as_str()).map(String::from));
    }
    Some(ports)
}

/// For each node, every node that runs before it (along edges).
fn ancestors(flow: &Flow) -> BTreeMap<&str, BTreeSet<&str>> {
    let mut parents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for e in &flow.edges {
        parents.entry(e.to.node.as_str()).or_default().push(e.from.node.as_str());
    }
    flow.nodes
        .iter()
        .map(|n| {
            let mut seen = BTreeSet::new();
            let mut stack = parents.get(n.id.as_str()).cloned().unwrap_or_default();
            while let Some(p) = stack.pop() {
                if seen.insert(p) {
                    stack.extend(parents.get(p).into_iter().flatten());
                }
            }
            (n.id.as_str(), seen)
        })
        .collect()
}

/// Node ids referenced as `nodes.<id>` or `nodes["<id>"]` in any expression inside `v`.
fn references(v: &Value) -> BTreeSet<String> {
    match v {
        Value::String(s) if il_core::expr::is_expression(s) => {
            s.match_indices("nodes").filter_map(|(i, _)| reference(&s[i + 5..])).collect()
        }
        Value::Array(list) => list.iter().flat_map(references).collect(),
        Value::Object(map) => map.values().flat_map(references).collect(),
        _ => BTreeSet::new(),
    }
}

fn reference(rest: &str) -> Option<String> {
    let id_char = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    let id = if let Some(r) = rest.strip_prefix('.') {
        r.split(|c: char| !id_char(c)).next()?
    } else {
        let r = rest.strip_prefix('[')?.trim_start_matches(['"', '\'']);
        r.split(|c: char| !id_char(c)).next()?
    };
    (!id.is_empty()).then(|| id.to_string())
}

#[cfg(test)]
mod tests {
    use il_core::{Category, NodeManifest};
    use serde_json::json;

    use super::*;

    fn catalog() -> Catalog {
        Catalog::new([
            NodeManifest::new("trigger.manual", Category::Trigger, "Manual", "Start by hand.").trigger(),
            NodeManifest::new("ai.write", Category::Ai, "Write", "Writes.").settings_schema(json!({
                "type": "object", "required": ["instructions"],
                "properties": {"instructions": {"type": "string"}, "input": {"type": "string"}}
            })),
            NodeManifest::new("logic.switch", Category::Logic, "Switch", "Routes.")
                .outputs(&["default"])
                .dynamic_outputs("cases"),
        ])
    }

    fn flow(v: Value) -> Flow {
        let mut base =
            json!({"id": "f", "workspaceId": "w", "version": 1, "name": "F", "kind": "automation", "groups": []});
        base.as_object_mut().unwrap().extend(v.as_object().unwrap().clone());
        serde_json::from_value(base).unwrap()
    }

    fn edge(id: &str, from: &str, port: &str, to: &str) -> Value {
        json!({"id": id, "from": {"node": from, "port": port}, "to": {"node": to, "port": "in"}})
    }

    #[test]
    fn accepts_a_valid_flow() {
        let f = flow(json!({
            "nodes": [
                {"id": "n_go", "type": "trigger.manual", "name": "Go"},
                {"id": "n_route", "type": "logic.switch", "name": "Route", "settings": {"cases": [{"name": "spam"}]}},
                {"id": "n_a", "type": "ai.write", "name": "A", "settings": {"instructions": "Hi", "input": "{{ nodes.n_go.json.x }}"}}
            ],
            "edges": [edge("e1", "n_go", "out", "n_route"), edge("e2", "n_route", "spam", "n_a")]
        }));
        assert_eq!(validate(&f, &catalog()), vec![]);
    }

    #[test]
    fn reports_what_the_model_must_fix() {
        let f = flow(json!({
            "nodes": [
                {"id": "n_a", "type": "ai.write", "name": "A", "group": "g_x", "settings": {"input": "{{ nodes['n_b'].json }} {{ nodes.n_zz }}"}},
                {"id": "n_b", "type": "ai.write", "name": "B", "settings": {"instructions": "Hi"}},
                {"id": "n_c", "type": "ai.magic", "name": "C"}
            ],
            "edges": [edge("e1", "n_a", "nope", "n_b"), edge("e2", "n_b", "out", "n_a")]
        }));
        let text: Vec<String> = validate(&f, &catalog()).iter().map(ToString::to_string).collect();
        let expect = [
            "nodes.n_a: group `g_x` is not in `groups`",
            "nodes.n_a.settings/instructions: is required",
            "nodes.n_c: unknown node type `ai.magic`; use a type from the catalog",
            "nodes: the flow needs a trigger step to start it",
            "edges.e1: `n_a` has no output `nope`; outputs: out",
            "nodes.n_a: the flow loops back to this step; flows must not have cycles",
            "nodes.n_a.settings: references unknown step `n_zz`",
        ];
        for e in expect {
            assert!(text.contains(&e.to_string()), "missing {e:?} in {text:#?}");
        }
    }

    #[test]
    fn flags_references_to_later_steps() {
        let f = flow(json!({
            "nodes": [
                {"id": "n_go", "type": "trigger.manual", "name": "Go"},
                {"id": "n_a", "type": "ai.write", "name": "A", "settings": {"instructions": "{{ nodes.n_b.json.text }}"}},
                {"id": "n_b", "type": "ai.write", "name": "B", "settings": {"instructions": "Hi"}}
            ],
            "edges": [edge("e1", "n_go", "out", "n_a"), edge("e2", "n_a", "out", "n_b")]
        }));
        let issues = validate(&f, &catalog());
        assert_eq!(issues, vec![Issue::new("nodes.n_a.settings", "references `n_b`, which does not run before it")]);
    }

    #[test]
    fn q05_flags_agent_tools_missing_from_the_flow() {
        let catalog = Catalog::new([
            NodeManifest::new("trigger.manual", Category::Trigger, "Manual", "Start by hand.").trigger(),
            NodeManifest::new("ai.agent", Category::Ai, "Agent", "Use tools."),
        ]);
        let f = flow(json!({
            "nodes": [
                {"id":"start","type":"trigger.manual","name":"Start"},
                {"id":"agent","type":"ai.agent","name":"Agent",
                    "settings":{"instructions":"Help","tools":["get_invoice"]}}
            ],
            "edges": [edge("e1", "start", "out", "agent")]
        }));
        assert!(
            validate(&f, &catalog).contains(&Issue::new(
                "nodes.agent.settings.tools[0]",
                "tool `get_invoice` is unavailable in this flow"
            ))
        );
    }
}
