//! Step cards on the Building screen (A03): `build.node` events for each step, and the build-only
//! test-suite card for AI apps, which the saved flow never contains.

use il_core::events::{BuildNodeStatus, NodeStat};
use il_core::{BuildEventKind, Edge, Flow, Node, PortRef, Position};
use serde_json::json;

/// The id of the build-only test-suite card.
pub(super) const TESTS: &str = "n_tests";

/// A step card's state.
pub(super) fn node(
    status: BuildNodeStatus,
    subtitle: Option<String>,
    label: Option<String>,
    summary: Option<String>,
) -> BuildEventKind {
    BuildEventKind::Node { status, subtitle, label, stats: vec![], summary, icon: None, tone: None, progress: None }
}

/// The final state of every step after testing: fixed if a fix changed it, a warning if a problem
/// remains on it, otherwise ready.
pub(super) fn finals<'a>(
    flow: &'a Flow,
    fixed: &'a [String],
    remaining: &'a [Option<String>],
) -> impl Iterator<Item = (&'a str, BuildEventKind)> + 'a {
    flow.nodes.iter().map(move |n| {
        let event = if remaining.iter().any(|r| r.as_deref() == Some(n.id.as_str())) {
            node(BuildNodeStatus::Warning, None, Some("Check in the editor".into()), None)
        } else if fixed.contains(&n.id) {
            node(BuildNodeStatus::Fixed, None, Some("Fixed".into()), None)
        } else {
            node(BuildNodeStatus::Ready, None, None, None)
        };
        (n.id.as_str(), event)
    })
}

/// The test-suite card and its edge from the first agent, as JSON patches for the canvas.
pub(super) fn tests_card(flow: &Flow, count: usize) -> Vec<BuildEventKind> {
    let agent = flow.nodes.iter().find(|n| n.node_type == "ai.agent");
    let at = agent.map_or(Position::default(), |a| Position { x: a.position.x, y: a.position.y + 200.0 });
    let card = Node {
        id: TESTS.into(),
        node_type: "build.tests".into(),
        type_version: 1,
        name: format!("{count} tests"),
        position: at,
        group: agent.and_then(|a| a.group.clone()),
        settings: json!({}),
        credentials: Default::default(),
        disabled: false,
        notes: None,
    };
    let mut patches =
        vec![BuildEventKind::FlowPatch { json_patch: json!([{"op": "add", "path": "/nodes/-", "value": card}]) }];
    if let Some(agent) = agent {
        let edge = Edge {
            id: format!("e_{}_tests", agent.id),
            from: PortRef { node: agent.id.clone(), port: "bottom".into() },
            to: PortRef { node: TESTS.into(), port: "top".into() },
        };
        patches
            .push(BuildEventKind::FlowPatch { json_patch: json!([{"op": "add", "path": "/edges/-", "value": edge}]) });
    }
    patches
}

/// The test-suite card's state after a round: `passed` of `total`, and how many were fixed so far.
pub(super) fn tests_state(passed: u32, total: usize, fixed: usize, done: bool) -> BuildEventKind {
    let failed = (total as u32).saturating_sub(passed);
    let status = match (done, failed, fixed) {
        (false, _, _) => BuildNodeStatus::Building,
        (true, 0, 0) => BuildNodeStatus::Ready,
        (true, 0, _) => BuildNodeStatus::Fixed,
        _ => BuildNodeStatus::Warning,
    };
    let label = match status {
        BuildNodeStatus::Fixed => Some(format!("Fixed {fixed} issue{}", if fixed == 1 { "" } else { "s" })),
        BuildNodeStatus::Warning => Some(format!("{failed} failing")),
        BuildNodeStatus::Building if failed > 0 => Some(format!("Fixing {failed}…")),
        _ => None,
    };
    let mut stats = vec![NodeStat { label: "Passed".into(), value: format!("{passed} / {total}") }];
    if fixed > 0 {
        stats.push(NodeStat { label: "Fixed automatically".into(), value: fixed.to_string() });
    }
    BuildEventKind::Node {
        status,
        subtitle: Some("Test suite".into()),
        label,
        stats,
        summary: Some(format!("{passed} / {total} passed")),
        icon: Some("tests".into()),
        tone: Some("tests".into()),
        progress: None,
    }
}
