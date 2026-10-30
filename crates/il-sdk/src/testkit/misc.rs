//! Small fakes: knowledge, notifier, media, clock, progress, approvals.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use il_core::FileRef;
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::ctx::{ApprovalGate, ApprovalSpec, ProgressSink, RunInfo};
use crate::node::NodeError;
use crate::services::clock::Clock;
use crate::services::knowledge::{KnowledgeError, KnowledgeService, SearchHit, SourceInput};
use crate::services::media::{MediaError, MediaGen, MediaRequest, MediaResult};
use crate::services::notifier::{Notification, Notifier, NotifyError};

/// Returns scripted hits for every search.
#[derive(Default)]
pub struct FakeKnowledge {
    pub hits: Mutex<Vec<SearchHit>>,
}

#[async_trait]
impl KnowledgeService for FakeKnowledge {
    async fn create_base(&self, _ws: &str, _name: &str) -> Result<String, KnowledgeError> {
        Ok(crate::id("kb"))
    }
    async fn add_sources(&self, _kb: &str, sources: Vec<SourceInput>) -> Result<Vec<String>, KnowledgeError> {
        Ok(sources.iter().map(|_| crate::id("src")).collect())
    }
    async fn search(&self, _kb: &str, _q: &str, top_k: usize) -> Result<Vec<SearchHit>, KnowledgeError> {
        let hits = self.hits.lock().map(|h| h.clone()).unwrap_or_default();
        Ok(hits.into_iter().take(top_k).collect())
    }
    async fn parse(&self, file_id: &str, _format: Option<&str>) -> Result<Value, KnowledgeError> {
        Ok(json!({"text": format!("Parsed text of {file_id}"), "sections": []}))
    }
}

/// Records every notification.
#[derive(Default)]
pub struct FakeNotifier {
    pub sent: Mutex<Vec<Notification>>,
}

#[async_trait]
impl Notifier for FakeNotifier {
    async fn send(&self, n: &Notification) -> Result<Option<String>, NotifyError> {
        self.sent.lock().map_err(|_| NotifyError::Send("poisoned".into()))?.push(n.clone());
        Ok(Some(crate::id("msg")))
    }
}

/// Returns a placeholder file for every request (like the `recorded` provider).
#[derive(Default)]
pub struct FakeMedia;

#[async_trait]
impl MediaGen for FakeMedia {
    async fn generate(&self, req: MediaRequest) -> Result<MediaResult, MediaError> {
        let (mime, ext) = match req.kind {
            crate::services::MediaKind::Video => ("video/mp4", "mp4"),
            crate::services::MediaKind::Voice => ("audio/mpeg", "mp3"),
            crate::services::MediaKind::Image => ("image/png", "png"),
        };
        Ok(MediaResult {
            file: FileRef {
                id: crate::id("file"),
                mime: mime.into(),
                bytes: 0,
                name: format!("fake.{ext}"),
                preview_url: None,
            },
            cost_usd: 0.0,
            // A step's own key shows as its provider, so tests can see it was passed on.
            provider: req.key.map_or_else(|| "fake".into(), |key| key.provider),
        })
    }
}

/// A clock that only moves when advanced. `sleep` waits until the fake time passes the deadline.
pub struct FakeClock {
    now: watch::Sender<DateTime<Utc>>,
}

impl Default for FakeClock {
    fn default() -> Self {
        let start = Utc.with_ymd_and_hms(2026, 10, 26, 9, 0, 0).single().unwrap_or_else(Utc::now);
        Self { now: watch::channel(start).0 }
    }
}

impl FakeClock {
    /// Sets the time.
    pub fn set(&self, t: DateTime<Utc>) {
        self.now.send_replace(t);
    }
    /// Moves time forward.
    pub fn advance(&self, d: Duration) {
        let next = *self.now.borrow() + chrono::Duration::from_std(d).unwrap_or_default();
        self.now.send_replace(next);
    }
}

#[async_trait]
impl Clock for FakeClock {
    fn now(&self) -> DateTime<Utc> {
        *self.now.borrow()
    }
    async fn sleep(&self, d: Duration) {
        let deadline = self.now() + chrono::Duration::from_std(d).unwrap_or_default();
        let mut rx = self.now.subscribe();
        while *rx.borrow_and_update() < deadline {
            if rx.changed().await.is_err() {
                return;
            }
        }
    }
}

/// Ignores progress.
#[derive(Default)]
pub struct NoopProgress {
    pub events: Mutex<Vec<(f32, String)>>,
}

impl ProgressSink for NoopProgress {
    fn progress(&self, _run: &RunInfo, pct: f32, message: &str) {
        if let Ok(mut e) = self.events.lock() {
            e.push((pct, message.into()));
        }
    }
}

/// Records approval requests and returns new ids.
#[derive(Default)]
pub struct FakeApprovals {
    pub requested: Mutex<Vec<ApprovalSpec>>,
}

#[async_trait]
impl ApprovalGate for FakeApprovals {
    async fn request(&self, _run: &RunInfo, spec: ApprovalSpec) -> Result<String, NodeError> {
        self.requested.lock().map_err(|_| NodeError::Failed("poisoned".into()))?.push(spec);
        Ok(crate::id("apr"))
    }
}
