//! Service traits (PRD §7.2). Engine, architect and publish reach storage, models, files,
//! knowledge, runs and each other only through these.

pub mod architect;
pub mod clock;
pub mod files;
pub mod knowledge;
pub mod media;
pub mod metering;
pub mod models;
pub mod notifier;
pub mod runner;
pub mod store;
pub mod tools;

use std::sync::Arc;

pub use architect::ArchitectService;
pub use clock::{Clock, SystemClock};
pub use files::{FileError, FileStore};
pub use knowledge::{KnowledgeError, KnowledgeService, SearchHit};
pub use media::{MediaError, MediaGen, MediaKey, MediaKind, MediaRequest, MediaResult};
pub use models::{ModelError, ModelGateway, ModelProvider, ModelRole};
pub use notifier::{Notification, Notifier, NotifyError};
pub use runner::{FlowActivation, FlowRunner, NoActivation, RunnerError, StartRun};
pub use store::{Credential, Store, StoreError};
pub use tools::{AgentTool, FlowToolProvider, NoFlowTools, ToolError};

use crate::http::SafeHttp;

/// The bundle of services handed to nodes (via `NodeCtx`) and to route factories.
#[derive(Clone)]
pub struct Services {
    pub store: Arc<dyn Store>,
    pub models: Arc<dyn ModelGateway>,
    pub files: Arc<dyn FileStore>,
    pub knowledge: Arc<dyn KnowledgeService>,
    pub notifier: Arc<dyn Notifier>,
    pub media: Arc<dyn MediaGen>,
    pub clock: Arc<dyn Clock>,
    pub http: SafeHttp,
    /// Tools available to Agent nodes, supplied by connector and node adapters.
    pub tools: Vec<Arc<dyn AgentTool>>,
    /// Each flow's own tools (its OpenAPI operations), resolved when a run starts.
    pub flow_tools: Arc<dyn FlowToolProvider>,
}
