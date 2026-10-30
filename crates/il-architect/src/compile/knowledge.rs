//! Knowledge steps across a revision: steps the agents searched survive, and a step the model adds
//! in place of one it was shown is folded back into that one.

use il_core::Flow;
use serde_json::{Value, json};

const STEP: &str = "knowledge.base";

/// Puts back knowledge steps that agents searched before a revision and the model left out: models
/// rewriting a flow drop steps no edge leads to, and the agents are left with nothing to search.
/// Returns whether it put any back.
pub(super) fn keep(before: &Flow, next: &mut Flow) -> bool {
    let mut kept = false;
    for step in before.nodes.iter().filter(|n| n.node_type == STEP) {
        let agents: Vec<&str> = before
            .nodes
            .iter()
            .filter(|n| n.node_type == "ai.agent")
            .filter(|n| n.settings["knowledgeBases"].as_array().is_some_and(|b| b.iter().any(|v| v == &step.id)))
            .map(|n| n.id.as_str())
            .collect();
        if next.node(&step.id).is_some() || agents.is_empty() {
            continue;
        }
        kept = true;
        next.nodes.push(step.clone());
        for agent in agents {
            if let Some(kept) = next.nodes.iter_mut().find(|n| n.id == agent && n.node_type == "ai.agent") {
                let mut bases = kept.settings["knowledgeBases"].as_array().cloned().unwrap_or_default();
                if !bases.contains(&json!(step.id)) {
                    bases.push(json!(step.id));
                }
                kept.settings["knowledgeBases"] = json!(bases);
            }
        }
        let linked = before.edges.iter().filter(|e| e.from.node == step.id && next.node(&e.to.node).is_some());
        next.edges.extend(linked.cloned().collect::<Vec<_>>());
    }
    kept
}

/// Folds knowledge steps that `next` adds into the first step it kept from `before`: their sources
/// join that step, and agents and edges point at it. Fixing a failed test, models often write a
/// fresh knowledge step beside the one holding the attachments, and the flow ends up with two.
pub(super) fn fold_new(before: &Flow, next: &mut Flow) {
    let Some(target) = next.nodes.iter().position(|n| n.node_type == STEP && before.node(&n.id).is_some()) else {
        return;
    };
    let target_id = next.nodes[target].id.clone();
    let added: Vec<String> = next
        .nodes
        .iter()
        .filter(|n| n.node_type == STEP && before.node(&n.id).is_none())
        .map(|n| n.id.clone())
        .collect();
    for id in &added {
        let sources = next.node(id).and_then(|n| n.settings["sources"].as_array().cloned()).unwrap_or_default();
        let step = &mut next.nodes[target];
        let mut merged = step.settings["sources"].as_array().cloned().unwrap_or_default();
        for source in sources {
            if !merged.iter().any(|m| m["value"] == source["value"]) {
                merged.push(source);
            }
        }
        step.settings["sources"] = Value::Array(merged);
        next.nodes.retain(|n| &n.id != id);
        for agent in next.nodes.iter_mut().filter(|n| n.node_type == "ai.agent") {
            if let Some(bases) = agent.settings["knowledgeBases"].as_array() {
                let mut ids: Vec<Value> = Vec::new();
                for base in bases.iter().map(|b| if b == id.as_str() { json!(target_id) } else { b.clone() }) {
                    if !ids.contains(&base) {
                        ids.push(base);
                    }
                }
                agent.settings["knowledgeBases"] = Value::Array(ids);
            }
        }
        for edge in next.edges.iter_mut().filter(|e| &e.from.node == id) {
            edge.from.node = target_id.clone();
        }
        let mut seen = Vec::new();
        next.edges.retain(|e| {
            let key = (e.from.clone(), e.to.clone());
            let fresh = !seen.contains(&key);
            seen.push(key);
            fresh
        });
    }
}

#[cfg(test)]
mod tests {
    use il_core::Flow;
    use serde_json::json;

    use super::{fold_new, keep};

    fn flow(nodes: serde_json::Value, edges: serde_json::Value) -> Flow {
        serde_json::from_value(json!({"id": "flw_1", "workspaceId": "ws", "version": 1, "name": "Help",
            "kind": "app", "nodes": nodes, "edges": edges}))
        .unwrap()
    }

    #[test]
    fn q22_a_revision_keeps_the_knowledge_step_its_agents_search() {
        let before = flow(
            json!([
                {"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"knowledgeBases": ["n_kb"]}},
                {"id": "n_kb", "type": "knowledge.base", "name": "Docs", "settings": {"name": "docs"}},
                {"id": "n_unused", "type": "knowledge.base", "name": "Old", "settings": {"name": "old"}}
            ]),
            json!([]),
        );
        let mut next = flow(
            json!([{"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"knowledgeBases": []}}]),
            json!([]),
        );
        assert!(keep(&before, &mut next));
        assert!(next.node("n_kb").is_some());
        assert!(next.node("n_unused").is_none(), "a base no agent searched may go");
        assert_eq!(next.node("n_agent").unwrap().settings["knowledgeBases"], json!(["n_kb"]));
    }

    #[test]
    fn q29_a_knowledge_step_added_by_a_fix_joins_the_existing_one() {
        let kb = |id: &str, url: &str| {
            json!({"id": id, "type": "knowledge.base", "name": id, "settings": {"name": id,
                "sources": [{"kind": "file", "value": "fil_1"}, {"kind": "url", "value": url}]}})
        };
        let agent = |bases: serde_json::Value| json!({"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {"knowledgeBases": bases}});
        let edge = |from: &str| json!({"id": format!("e_{from}"), "from": {"node": from, "port": "kb"}, "to": {"node": "n_agent", "port": "kb"}});
        let before =
            flow(json!([agent(json!(["n_kb1"])), kb("n_kb1", "https://help.example")]), json!([edge("n_kb1")]));
        let mut next = flow(
            json!([
                agent(json!(["n_kb1", "n_docs"])),
                kb("n_kb1", "https://help.example"),
                kb("n_docs", "https://faq.example")
            ]),
            json!([edge("n_kb1"), edge("n_docs")]),
        );
        fold_new(&before, &mut next);
        assert!(next.node("n_docs").is_none());
        assert_eq!(next.node("n_agent").unwrap().settings["knowledgeBases"], json!(["n_kb1"]));
        let sources = &next.node("n_kb1").unwrap().settings["sources"];
        assert_eq!(sources.as_array().unwrap().len(), 3, "{sources}");
        assert_eq!(next.edges.len(), 1);
    }

    #[test]
    fn q29_with_no_knowledge_step_before_a_new_one_stays() {
        let before = flow(json!([{"id": "n_agent", "type": "ai.agent", "name": "Agent", "settings": {}}]), json!([]));
        let mut next = flow(
            json!([{"id": "n_kb", "type": "knowledge.base", "name": "Docs", "settings": {"name": "docs"}}]),
            json!([]),
        );
        fold_new(&before, &mut next);
        assert!(next.node("n_kb").is_some());
    }
}
