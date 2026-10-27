//! Ordered event emission to the store and a live broadcast bus (EN-4).
//!
//! Every run event gets the next `seq` for its run, is appended to the store, then broadcast.
//! SSE handlers read history from the store (`after` cursor) and follow live events from
//! [`EventBus::subscribe`].

use std::collections::HashMap;
use std::sync::Arc;

use il_core::{RunEvent, RunEventKind};
use il_sdk::services::{Clock, Store};
use tokio::sync::{Mutex, broadcast};

/// Live + persisted run events.
#[derive(Clone)]
pub struct EventBus {
    store: Arc<dyn Store>,
    clock: Arc<dyn Clock>,
    tx: broadcast::Sender<RunEvent>,
    seqs: Arc<Mutex<HashMap<String, u64>>>,
}

impl EventBus {
    /// Creates a bus over the store.
    pub fn new(store: Arc<dyn Store>, clock: Arc<dyn Clock>) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { store, clock, tx, seqs: Arc::new(Mutex::new(HashMap::new())) }
    }

    /// Follows live events for all runs (filter by `run_id`).
    pub fn subscribe(&self) -> broadcast::Receiver<RunEvent> {
        self.tx.subscribe()
    }

    /// Emits one event: assigns the next seq, persists, broadcasts. Store errors are logged
    /// (the run keeps going) and the event is still broadcast.
    pub async fn emit(&self, run_id: &str, node_id: Option<&str>, kind: RunEventKind) -> RunEvent {
        let mut seqs = self.seqs.lock().await;
        let next = match seqs.get(run_id) {
            Some(s) => s + 1,
            None => self.last_seq(run_id).await + 1,
        };
        seqs.insert(run_id.to_string(), next);
        let event = RunEvent {
            seq: next,
            ts: self.clock.now(),
            run_id: run_id.to_string(),
            node_id: node_id.map(str::to_string),
            kind,
        };
        if let Err(e) = self.store.append_run_event(&event).await {
            tracing::error!(run_id, error = %e, "failed to persist run event");
        }
        drop(seqs);
        let _ = self.tx.send(event.clone());
        event
    }

    async fn last_seq(&self, run_id: &str) -> u64 {
        self.store.run_events(run_id, 0).await.ok().and_then(|v| v.last().map(|e| e.seq)).unwrap_or(0)
    }
}
