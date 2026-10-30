//! Layered auto-layout with groups: each step goes one column right of its furthest parent, and
//! steps in a column are ordered by their parents' rows, then by group, so stages stay together.

use std::collections::BTreeMap;

use il_core::{Flow, Position};

/// Column pitch: the 264 px node card plus a 40 px gap (matches the web canvas).
pub const COLUMN: f64 = 304.0;
/// Row pitch within a column.
pub const ROW: f64 = 200.0;
/// The y of a single-row column, so one straight line of steps sits at the same height.
const MIDDLE: f64 = 200.0;

/// Positions for a revised flow: steps keep where they were (the person may have moved them),
/// unless the revision added steps, which fit only into a fresh layout of the whole flow — a new
/// step's place next to old positions overlapped them.
pub fn settle(before: &Flow, after: &mut Flow) {
    if after.nodes.iter().any(|n| before.node(&n.id).is_none()) {
        apply(after);
        return;
    }
    for node in &mut after.nodes {
        if let Some(old) = before.node(&node.id) {
            node.position = old.position;
        }
    }
}

/// Sets every node's position.
pub fn apply(flow: &mut Flow) {
    let index: BTreeMap<&str, usize> = flow.nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
    let mut parents = vec![Vec::new(); flow.nodes.len()];
    for e in &flow.edges {
        if let (Some(&f), Some(&t)) = (index.get(e.from.node.as_str()), index.get(e.to.node.as_str()))
            && f != t
        {
            parents[t].push(f);
        }
    }
    let layer = layers(&parents);
    let group = |i: usize| {
        let g = flow.nodes[i].group.as_deref();
        flow.groups.iter().position(|x| Some(x.id.as_str()) == g).unwrap_or(usize::MAX)
    };
    let mut row = vec![0.0_f64; flow.nodes.len()];
    let columns = layer.iter().copied().max().map_or(0, |m| m + 1);
    for col in 0..columns {
        let mut members: Vec<usize> = (0..flow.nodes.len()).filter(|&i| layer[i] == col).collect();
        let center = |i: usize| {
            let placed: Vec<f64> = parents[i].iter().filter(|&&p| layer[p] < col).map(|&p| row[p]).collect();
            if placed.is_empty() { f64::MAX } else { placed.iter().sum::<f64>() / placed.len() as f64 }
        };
        members.sort_by(|&a, &b| center(a).total_cmp(&center(b)).then(group(a).cmp(&group(b))).then(a.cmp(&b)));
        let mid = (members.len() as f64 - 1.0) / 2.0;
        for (k, &i) in members.iter().enumerate() {
            row[i] = k as f64 - mid;
        }
    }
    for (i, n) in flow.nodes.iter_mut().enumerate() {
        n.position = Position { x: layer[i] as f64 * COLUMN, y: MIDDLE + row[i] * ROW };
    }
}

/// Longest-path layering. Steps caught in a cycle go one column after their placed parents.
fn layers(parents: &[Vec<usize>]) -> Vec<usize> {
    let n = parents.len();
    let mut layer: Vec<Option<usize>> = vec![None; n];
    loop {
        let mut progressed = false;
        for i in 0..n {
            if layer[i].is_none() && parents[i].iter().all(|&p| layer[p].is_some()) {
                layer[i] = Some(parents[i].iter().filter_map(|&p| layer[p]).map(|l| l + 1).max().unwrap_or(0));
                progressed = true;
            }
        }
        if !progressed {
            let Some(stuck) = (0..n).find(|&i| layer[i].is_none()) else { break };
            layer[stuck] = Some(parents[stuck].iter().filter_map(|&p| layer[p]).map(|l| l + 1).max().unwrap_or(0));
        }
    }
    layer.into_iter().map(Option::unwrap_or_default).collect()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn flow(nodes: &[(&str, &str)], edges: &[(&str, &str)]) -> Flow {
        serde_json::from_value(json!({
            "id": "f", "workspaceId": "w", "version": 1, "name": "F", "kind": "automation",
            "groups": [{"id": "g1", "label": "One"}, {"id": "g2", "label": "Two"}],
            "nodes": nodes.iter().map(|(id, g)| json!({"id": id, "type": "x", "name": id, "group": g})).collect::<Vec<_>>(),
            "edges": edges.iter().enumerate().map(|(i, (f, t))| json!({
                "id": format!("e{i}"), "from": {"node": f, "port": "out"}, "to": {"node": t, "port": "in"}
            })).collect::<Vec<_>>()
        }))
        .unwrap()
    }

    fn at(f: &Flow, id: &str) -> (f64, f64) {
        let p = &f.node(id).unwrap().position;
        (p.x, p.y)
    }

    #[test]
    fn lays_out_columns_by_longest_path() {
        let mut f = flow(
            &[("a", "g1"), ("b", "g1"), ("c", "g2"), ("d", "g2")],
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")],
        );
        apply(&mut f);
        assert_eq!(at(&f, "a"), (0.0, 200.0));
        assert_eq!(at(&f, "b"), (304.0, 100.0));
        assert_eq!(at(&f, "c"), (304.0, 300.0));
        assert_eq!(at(&f, "d"), (608.0, 200.0));
    }

    #[test]
    fn survives_cycles() {
        let mut f = flow(&[("a", "g1"), ("b", "g1")], &[("a", "b"), ("b", "a")]);
        apply(&mut f);
        assert_ne!(at(&f, "a"), at(&f, "b"));
    }

    #[test]
    fn q27_revisions_keep_places_unless_they_add_steps() {
        let mut before = flow(&[("a", "g1"), ("b", "g2")], &[("a", "b")]);
        before.nodes[1].position = Position { x: 900.0, y: 50.0 };
        let mut edited = before.clone();
        edited.nodes[1].name = "renamed".into();
        edited.nodes[1].position = Position { x: 0.0, y: 0.0 };
        settle(&before, &mut edited);
        assert_eq!(at(&edited, "b"), (900.0, 50.0), "a moved step stays where the person put it");

        let mut grown = flow(&[("a", "g1"), ("c", "g1"), ("b", "g2")], &[("a", "c"), ("c", "b")]);
        settle(&before, &mut grown);
        let places: Vec<_> = ["a", "c", "b"].iter().map(|id| at(&grown, id)).collect();
        assert!(places[0].0 < places[1].0 && places[1].0 < places[2].0, "a new step gets its own column: {places:?}");
    }
}
