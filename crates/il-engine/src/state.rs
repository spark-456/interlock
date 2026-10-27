//! The run checkpoint stored in `Run.state`. Everything needed to continue a run after a
//! restart or a human decision lives here.

use std::collections::{BTreeMap, BTreeSet};

use il_core::Item;
use serde::{Deserialize, Serialize};

/// Output items by port.
pub type Ports = BTreeMap<String, Vec<Item>>;

/// Graph progress of one run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint {
    /// The trigger node the run started from.
    pub trigger_node: String,
    /// Finished nodes and their outputs.
    pub outputs: BTreeMap<String, Ports>,
    /// Nodes on branches that were not taken.
    pub skipped: BTreeSet<String>,
    /// Nodes paused for human decisions.
    pub partial: BTreeMap<String, Partial>,
    /// Response items captured from Respond nodes (chat/API/MCP invocations).
    pub response: Vec<Item>,
}

/// A node with at least one item waiting for an approval.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Partial {
    /// The node's full input.
    pub input: Ports,
    /// True for batch nodes (one call with all items).
    pub batch: bool,
    /// Finished per-item (or batch = index 0) outputs.
    pub results: BTreeMap<usize, Ports>,
    /// approval id → item index still waiting.
    pub waiting: BTreeMap<String, usize>,
    pub cost_usd: f64,
    pub started_ms: i64,
}

impl Checkpoint {
    /// Whether a node has finished or was skipped.
    pub fn resolved(&self, node_id: &str) -> bool {
        self.outputs.contains_key(node_id) || self.skipped.contains(node_id)
    }

    /// Reads the checkpoint from a run's opaque state.
    pub fn from_state(state: &serde_json::Value) -> Self {
        serde_json::from_value(state.clone()).unwrap_or_default()
    }

    /// Serializes for `Run.state`.
    pub fn to_state(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

impl Partial {
    /// Merges finished results in item order into one set of ports.
    pub fn merged(&self) -> Ports {
        let mut out = Ports::new();
        for ports in self.results.values() {
            for (port, items) in ports {
                out.entry(port.clone()).or_default().extend(items.iter().cloned());
            }
        }
        out
    }
}
