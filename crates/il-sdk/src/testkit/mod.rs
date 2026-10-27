//! Offline fakes for every service plus `NodeHarness`. Enable with the `testkit` feature.

mod files;
mod gateway;
mod harness;
mod misc;
mod store;

use std::sync::Arc;

pub use files::FakeFiles;
pub use gateway::FakeGateway;
pub use harness::{NodeHarness, assert_manifest_valid};
pub use misc::{FakeApprovals, FakeClock, FakeKnowledge, FakeMedia, FakeNotifier, NoopProgress};
pub use store::MemoryStore;

use crate::http::SafeHttp;
use crate::services::Services;

/// A full set of fake services. Fields are public so tests can keep handles to the fakes.
#[derive(Clone)]
pub struct FakeServices {
    pub store: Arc<MemoryStore>,
    pub models: Arc<FakeGateway>,
    pub files: Arc<FakeFiles>,
    pub knowledge: Arc<FakeKnowledge>,
    pub notifier: Arc<FakeNotifier>,
    pub media: Arc<FakeMedia>,
    pub clock: Arc<FakeClock>,
}

impl Default for FakeServices {
    fn default() -> Self {
        Self {
            store: Arc::new(MemoryStore::default()),
            models: Arc::new(FakeGateway::default()),
            files: Arc::new(FakeFiles::default()),
            knowledge: Arc::new(FakeKnowledge::default()),
            notifier: Arc::new(FakeNotifier::default()),
            media: Arc::new(FakeMedia),
            clock: Arc::new(FakeClock::default()),
        }
    }
}

impl FakeServices {
    /// The `Services` bundle backed by these fakes. HTTP allows loopback for wiremock.
    pub fn services(&self) -> Services {
        Services {
            store: self.store.clone(),
            models: self.models.clone(),
            files: self.files.clone(),
            knowledge: self.knowledge.clone(),
            notifier: self.notifier.clone(),
            media: self.media.clone(),
            clock: self.clock.clone(),
            http: SafeHttp::permissive_for_tests(),
        }
    }
}

/// Fake HTTP: a client that may reach local mock servers (use `wiremock` for responses).
pub fn fake_http() -> SafeHttp {
    SafeHttp::permissive_for_tests()
}
