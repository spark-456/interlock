//! `NodeHarness`: run one node with fake services, rendering settings like the engine does.

use std::collections::BTreeMap;
use std::sync::Arc;

use il_core::expr::{Scope, render_value};
use il_core::{ExecMode, Item};
use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::{FakeApprovals, FakeServices, NoopProgress};
use crate::ctx::{NodeCtx, RunInfo};
use crate::node::{NodeError, NodeExecutor, NodeInput, NodeOutput};

/// Builder for a single-node test run.
pub struct NodeHarness<N: NodeExecutor> {
    node: N,
    settings: Value,
    input: Vec<Item>,
    credentials: BTreeMap<String, String>,
    /// The fakes; tweak them (script the gateway, seed credentials) before `run`.
    pub fakes: FakeServices,
    pub progress: Arc<NoopProgress>,
    pub approvals: Arc<FakeApprovals>,
}

impl<N: NodeExecutor> NodeHarness<N> {
    /// Starts a harness for `node`.
    pub fn new(node: N) -> Self {
        Self {
            node,
            settings: Value::Object(Default::default()),
            input: vec![],
            credentials: BTreeMap::new(),
            fakes: FakeServices::default(),
            progress: Arc::new(NoopProgress::default()),
            approvals: Arc::new(FakeApprovals::default()),
        }
    }
    /// Raw settings (expressions allowed).
    pub fn settings(mut self, settings: Value) -> Self {
        self.settings = settings;
        self
    }
    /// Input items from JSON values.
    pub fn input_json(mut self, items: Vec<Value>) -> Self {
        self.input = items.into_iter().map(Item::json).collect();
        self
    }
    /// Input items.
    pub fn input(mut self, items: Vec<Item>) -> Self {
        self.input = items;
        self
    }
    /// Binds a credential key to a credential id (seed it in `fakes.store`).
    pub fn credential(mut self, key: &str, id: &str) -> Self {
        self.credentials.insert(key.into(), id.into());
        self
    }

    /// Validates settings, then executes per item (merging outputs) or once for batch nodes.
    pub async fn run(&self) -> Result<NodeOutput, NodeError> {
        self.node.validate(&self.settings).map_err(NodeError::InvalidSettings)?;
        let all: Vec<Value> = self.input.iter().map(|i| i.json.clone()).collect();
        let per_item = self.node.manifest().exec_mode == ExecMode::PerItem && !self.input.is_empty();
        let batches: Vec<(usize, Vec<Item>)> = if per_item {
            self.input.iter().cloned().enumerate().map(|(i, it)| (i, vec![it])).collect()
        } else {
            vec![(0, self.input.clone())]
        };
        let mut merged = NodeOutput::default();
        for (index, items) in batches {
            let current = items.first().map(|i| i.json.clone()).unwrap_or(Value::Null);
            let scope = Scope::new().item(current).items(all.clone());
            let settings = render_value(&self.settings, &scope).map_err(|e| NodeError::Failed(e.to_string()))?;
            let mut ctx = NodeCtx {
                settings,
                run: RunInfo {
                    run_id: "run_test".into(),
                    flow_id: "flw_test".into(),
                    workspace_id: "ws_test".into(),
                    node_id: "n_test".into(),
                    node_name: self.node.manifest().title.clone(),
                    trigger: "test".into(),
                    attempt: 1,
                    item_index: index,
                    test: true,
                },
                services: self.fakes.services(),
                credential_ids: self.credentials.clone(),
                cancel: CancellationToken::new(),
                progress: self.progress.clone(),
                approvals: self.approvals.clone(),
                resume: None,
            };
            let out = self.node.execute(&mut ctx, NodeInput::items(items)).await?;
            for (port, items) in out.ports {
                merged.ports.entry(port).or_default().extend(items);
            }
            merged.cost_usd += out.cost_usd;
            if out.waiting_for.is_some() {
                merged.waiting_for = out.waiting_for;
            }
        }
        Ok(merged)
    }
}

/// Checks a manifest is well-formed: type id shape, object schema, triggers have no inputs,
/// and every example validates against the schema.
pub fn assert_manifest_valid(node: &dyn NodeExecutor) {
    let m = node.manifest();
    let id = &m.type_id;
    assert!(
        id.split('.').count() >= 2
            && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_'),
        "bad type id `{id}`"
    );
    assert_eq!(m.settings_schema.get("type").and_then(Value::as_str), Some("object"), "{id}: schema must be an object");
    assert!(!m.description.is_empty(), "{id}: description required");
    if m.trigger {
        assert!(m.inputs.is_empty(), "{id}: triggers have no inputs");
    }
    assert!(!m.examples.is_empty(), "{id}: at least one example");
    for ex in &m.examples {
        if let Err(issues) = node.validate(&ex.settings) {
            panic!("{id}: example `{}` is invalid: {issues:?}", ex.title);
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use il_core::{Category, NodeManifest};
    use serde_json::json;

    use super::*;
    use crate::registry::NodeFactory;

    struct Echo(NodeManifest);
    impl Default for Echo {
        fn default() -> Self {
            Self(
                NodeManifest::new("test.echo", Category::Data, "Echo", "Copies `text` into the item.")
                    .settings_schema(
                        json!({"type": "object", "required": ["text"], "properties": {"text": {"type": "string"}}}),
                    )
                    .example("Greeting", json!({"text": "hi"})),
            )
        }
    }

    #[async_trait]
    impl NodeExecutor for Echo {
        fn manifest(&self) -> &NodeManifest {
            &self.0
        }
        async fn execute(&self, ctx: &mut NodeCtx, input: NodeInput) -> Result<NodeOutput, NodeError> {
            ctx.emit_progress(50.0, "echoing");
            let mut item = input.main().first().cloned().unwrap_or_default();
            item.json["echo"] = ctx.settings["text"].clone();
            Ok(NodeOutput::out(vec![item]))
        }
    }

    inventory::submit! { NodeFactory::new(|| Box::new(Echo::default())) }

    #[tokio::test]
    async fn harness_renders_per_item_and_merges() {
        let h = NodeHarness::new(Echo::default())
            .settings(json!({"text": "Hello {{ item.name }}"}))
            .input_json(vec![json!({"name": "Ana"}), json!({"name": "Bo"})]);
        let out = h.run().await.unwrap();
        let echoes: Vec<_> = out.ports["out"].iter().map(|i| i.json["echo"].clone()).collect();
        assert_eq!(echoes, [json!("Hello Ana"), json!("Hello Bo")]);
        assert_eq!(h.progress.events.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn harness_rejects_invalid_settings() {
        let err = NodeHarness::new(Echo::default()).run().await.unwrap_err();
        assert!(matches!(err, NodeError::InvalidSettings(_)));
    }

    #[test]
    fn registry_collects_submitted_nodes() {
        assert!(crate::registry::node("test.echo").is_some());
        assert_manifest_valid(crate::registry::node("test.echo").unwrap().as_ref());
    }
}
