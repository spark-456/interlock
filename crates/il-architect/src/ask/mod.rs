//! Ask Interlock (AR-8): a change request becomes a revised flow, returned as a JSON-Patch diff
//! with a short summary ("+1 step, 1 changed") for the editor to accept or discard.

use il_core::dto::AskDiff;
use il_core::{Flow, Node};
use il_sdk::services::models::ModelGateway;
use serde_json::{Value, json};

use crate::catalog::Catalog;
use crate::compile::{CompileError, revise};

/// Turns `message` into a diff against `flow`; also returns what the model calls cost.
pub async fn ask(
    models: &dyn ModelGateway,
    catalog: &Catalog,
    flow: &Flow,
    message: &str,
) -> Result<(AskDiff, f64), CompileError> {
    let revised = revise(models, catalog, flow, &format!("Change request: {message}")).await?;
    Ok((diff(flow, &revised.flow), revised.cost_usd))
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
    AskDiff { summary: summary(before, after, !ops.is_empty()), json_patch: Value::Array(ops) }
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
