//! Ask Interlock (AR-8): a change request becomes a revised flow, returned as a JSON-Patch diff
//! with a short summary ("+1 step, 1 changed") for the editor to accept or discard.

use il_core::dto::AskDiff;
use il_core::{Flow, Node};
use il_sdk::services::models::ModelGateway;
use serde_json::{Value, json};

use crate::catalog::Catalog;
use crate::compile::{CompileError, revise};

/// Turns `message` into a diff against `flow`, about the `focus` steps when the person pointed at
/// some; also returns what the model calls cost.
pub async fn ask(
    models: &dyn ModelGateway,
    catalog: &Catalog,
    flow: &Flow,
    message: &str,
    focus: &[String],
) -> Result<(AskDiff, f64), CompileError> {
    let mut revised = revise(models, catalog, flow, &request(flow, message, focus)).await?;
    keep_unfocused(flow, &mut revised.flow, focus);
    let mut diff = diff(flow, &revised.flow);
    diff.notes = revised.notes;
    Ok((diff, revised.cost_usd))
}

/// The change request for the model, naming the steps the person pointed at.
fn request(flow: &Flow, message: &str, focus: &[String]) -> String {
    let steps: Vec<String> = focus
        .iter()
        .filter_map(|id| flow.node(id))
        .map(|n| format!("\"{}\" (`{}`, {})", n.name, n.id, n.node_type))
        .collect();
    if steps.is_empty() {
        return format!("Change request: {message}");
    }
    format!(
        "Change request about the steps {}: {message}\n\nChange only those steps. You may add new steps and connect \
         them, but keep every other step exactly as it is.",
        steps.join(", ")
    )
}

/// When the person pointed at steps, every other step stays as it was: models asked about one step
/// still rewrite or drop others. New steps and the edges to them are kept.
fn keep_unfocused(before: &Flow, after: &mut Flow, focus: &[String]) {
    if !focus.iter().any(|id| before.node(id).is_some()) {
        return;
    }
    for old in before.nodes.iter().filter(|n| !focus.contains(&n.id)) {
        match after.nodes.iter_mut().find(|n| n.id == old.id) {
            Some(kept) => *kept = old.clone(),
            None => after.nodes.push(old.clone()),
        }
    }
    let same = |a: &il_core::Edge, b: &il_core::Edge| a.from == b.from && a.to == b.to;
    let fixed = |id: &str| before.node(id).is_some() && !focus.iter().any(|f| f == id);
    for edge in before.edges.iter().filter(|e| fixed(&e.from.node) && fixed(&e.to.node)) {
        if !after.edges.iter().any(|e| same(e, edge)) {
            after.edges.push(edge.clone());
        }
    }
}

/// The patch from `before` to `after`: whole-array replacements of whatever changed, which the
/// editor applies and then compares step by step.
pub fn diff(before: &Flow, after: &Flow) -> AskDiff {
    let mut ops = Vec::new();
    let mut replace = |path: &str, old: Value, new: Value| {
        if old != new {
            ops.push(json!({"op": "replace", "path": path, "value": new}));
        }
    };
    replace("/name", json!(before.name), json!(after.name));
    replace("/groups", json!(before.groups), json!(after.groups));
    replace("/nodes", json!(before.nodes), json!(after.nodes));
    replace("/edges", json!(before.edges), json!(after.edges));
    AskDiff { summary: summary(before, after, !ops.is_empty()), json_patch: Value::Array(ops), notes: Vec::new() }
}

/// Steps that differ in anything the person set (not position).
pub fn changed(before: &Node, after: &Node) -> bool {
    (&before.node_type, &before.name, &before.group, &before.settings, before.disabled)
        != (&after.node_type, &after.name, &after.group, &after.settings, after.disabled)
}

fn summary(before: &Flow, after: &Flow, any: bool) -> String {
    let added = after.nodes.iter().filter(|n| before.node(&n.id).is_none()).count();
    let removed = before.nodes.iter().filter(|n| after.node(&n.id).is_none()).count();
    let edited = after.nodes.iter().filter(|n| before.node(&n.id).is_some_and(|old| changed(old, n))).count();
    let steps = |n: usize| if n == 1 { "step" } else { "steps" };
    let mut parts = Vec::new();
    if added > 0 {
        parts.push(format!("+{added} {}", steps(added)));
    }
    if removed > 0 {
        parts.push(format!("−{removed} {}", steps(removed)));
    }
    if edited > 0 {
        parts.push(format!("{edited} changed"));
    }
    match (parts.is_empty(), any) {
        (false, _) => parts.join(", "),
        (true, true) => "Connections changed".into(),
        (true, false) => "No changes".into(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn q22_requests_name_the_steps_the_person_pointed_at() {
        let flow: il_core::Flow = serde_json::from_value(json!({"id": "flw_1", "workspaceId": "ws", "version": 1,
            "name": "Help", "kind": "app", "edges": [],
            "nodes": [{"id": "n_guard", "type": "ai.guardrails", "name": "Safety rules", "settings": {}}]}))
        .unwrap();
        let focused = super::request(&flow, "be stricter", &["n_guard".into(), "n_gone".into()]);
        assert!(
            focused
                .starts_with("Change request about the steps \"Safety rules\" (`n_guard`, ai.guardrails): be stricter")
        );
        assert_eq!(super::request(&flow, "be stricter", &[]), "Change request: be stricter");
    }

    fn flow(nodes: serde_json::Value, edges: serde_json::Value) -> il_core::Flow {
        serde_json::from_value(json!({"id": "flw_1", "workspaceId": "ws", "version": 1, "name": "Help", "kind": "app",
            "nodes": nodes, "edges": edges}))
        .unwrap()
    }

    fn edge(from: &str, to: &str) -> serde_json::Value {
        json!({"id": format!("e_{from}_{to}"), "from": {"node": from, "port": "out"}, "to": {"node": to, "port": "in"}})
    }

    #[test]
    fn q22_steps_the_person_did_not_point_at_stay_as_they_were() {
        let before = flow(
            json!([
                {"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"instructions": "Help."}},
                {"id": "n_guard", "type": "ai.guardrails", "name": "Safety rules", "settings": {"rules": []}},
                {"id": "n_kb", "type": "knowledge.base", "name": "Docs", "settings": {"name": "docs"}}
            ]),
            json!([edge("n_agent", "n_guard"), edge("n_kb", "n_agent")]),
        );
        let mut after = flow(
            json!([
                {"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"instructions": "Rewritten."}},
                {"id": "n_guard", "type": "ai.guardrails", "name": "Safety rules", "settings": {"rules": [{"name": "cancel"}]}},
                {"id": "n_review", "type": "human.approval", "name": "Review", "settings": {}}
            ]),
            json!([edge("n_guard", "n_review")]),
        );
        super::keep_unfocused(&before, &mut after, &["n_guard".into()]);
        assert_eq!(after.node("n_agent").unwrap().settings["instructions"], "Help.");
        assert_eq!(after.node("n_guard").unwrap().settings["rules"][0]["name"], "cancel");
        assert!(after.node("n_kb").is_some() && after.node("n_review").is_some());
        assert_eq!(after.edges.len(), 2, "the new edge stays and the dropped one between kept steps returns");
    }
}
