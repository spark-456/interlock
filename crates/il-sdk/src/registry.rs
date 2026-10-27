//! Self-registration (no central lists). Every node, trigger, model provider, media generator
//! and route group submits a factory with `inventory::submit!`; these functions collect them.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use il_core::NodeManifest;
use serde_json::Value;

use crate::node::NodeExecutor;
use crate::services::{
    FileStore, FlowToolProvider, MediaError, MediaGen, MediaKind, ModelError, ModelProvider, Services,
};

/// Registers a node type: `inventory::submit! { NodeFactory::new(|| Box::new(MyNode)) }`.
pub struct NodeFactory {
    make: fn() -> Box<dyn NodeExecutor>,
}

impl NodeFactory {
    /// Creates a factory.
    pub const fn new(make: fn() -> Box<dyn NodeExecutor>) -> Self {
        Self { make }
    }
}
inventory::collect!(NodeFactory);

/// Registers a model provider by config name (`tokenfactory`, `fake`).
pub struct ProviderFactory {
    pub name: &'static str,
    /// Builds the provider from its config section.
    pub make: fn(&Value) -> Result<Arc<dyn ModelProvider>, ModelError>,
}
inventory::collect!(ProviderFactory);

/// Builds a media generator with its config and shared file store.
pub type MediaGenBuilder = fn(&Value, Arc<dyn FileStore>) -> Result<Arc<dyn MediaGen>, MediaError>;

/// Registers a media generator by config name (`recorded`, …).
pub struct MediaGenFactory {
    pub name: &'static str,
    pub kinds: &'static [MediaKind],
    /// Builds the generator from its config section.
    pub make: MediaGenBuilder,
}
inventory::collect!(MediaGenFactory);

/// What a route group gets when mounted.
#[derive(Clone)]
pub struct RouteCtx {
    pub services: Services,
    pub runner: Arc<dyn crate::services::FlowRunner>,
    pub architect: Arc<dyn crate::services::ArchitectService>,
    /// Tools resolved for a published flow and authenticated workspace.
    pub flow_tools: Arc<dyn FlowToolProvider>,
    /// The whole config file as JSON; each module reads its own section.
    pub config: Value,
}

/// Registers an HTTP route group. Paths are absolute (`/v1/...`, `/api/oauth/...`, `/hooks/...`).
pub struct RouteFactory {
    pub name: &'static str,
    pub make: fn(&RouteCtx) -> axum::Router,
}
inventory::collect!(RouteFactory);

type NodeMap = BTreeMap<String, Arc<dyn NodeExecutor>>;

fn node_map() -> &'static NodeMap {
    static NODES: OnceLock<NodeMap> = OnceLock::new();
    NODES.get_or_init(|| {
        let mut map = NodeMap::new();
        for f in inventory::iter::<NodeFactory> {
            let node: Arc<dyn NodeExecutor> = Arc::from((f.make)());
            let id = node.manifest().type_id.clone();
            if map.insert(id.clone(), node).is_some() {
                tracing::error!(type_id = %id, "duplicate node type registered; keeping the last one");
            }
        }
        map
    })
}

/// All registered nodes, sorted by type id.
pub fn nodes() -> impl Iterator<Item = &'static Arc<dyn NodeExecutor>> {
    node_map().values()
}

/// A node by type id.
pub fn node(type_id: &str) -> Option<Arc<dyn NodeExecutor>> {
    node_map().get(type_id).cloned()
}

/// The catalog: every registered manifest, sorted by type id.
pub fn catalog() -> Vec<NodeManifest> {
    nodes().map(|n| n.manifest().clone()).collect()
}

/// Nodes that are triggers (derived from `NodeExecutor::as_trigger`).
pub fn triggers() -> impl Iterator<Item = &'static Arc<dyn NodeExecutor>> {
    nodes().filter(|n| n.as_trigger().is_some())
}

/// Registered model providers.
pub fn providers() -> impl Iterator<Item = &'static ProviderFactory> {
    inventory::iter::<ProviderFactory>.into_iter()
}

/// Registered media generators.
pub fn media_gens() -> impl Iterator<Item = &'static MediaGenFactory> {
    inventory::iter::<MediaGenFactory>.into_iter()
}

/// Registered route groups.
pub fn route_factories() -> impl Iterator<Item = &'static RouteFactory> {
    inventory::iter::<RouteFactory>.into_iter()
}
