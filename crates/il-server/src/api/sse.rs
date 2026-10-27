//! SSE helpers: history from the store after a cursor, then live events, with heartbeats.
//! The cursor comes from `?after=` or the `Last-Event-ID` header (EventSource reconnects).

use std::convert::Infallible;
use std::time::Duration;

use axum::http::HeaderMap;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::Stream;
use serde::Serialize;
use tokio::sync::mpsc;

/// Heartbeat interval for idle streams.
const HEARTBEAT: Duration = Duration::from_secs(15);

/// Resolves the resume cursor.
pub fn cursor(after: Option<u64>, headers: &HeaderMap) -> u64 {
    after
        .or_else(|| headers.get("last-event-id").and_then(|v| v.to_str().ok()).and_then(|s| s.parse().ok()))
        .unwrap_or(0)
}

/// One SSE frame with `id: <seq>` so reconnects resume exactly.
pub fn frame<T: Serialize>(seq: u64, event: &T) -> Event {
    Event::default().id(seq.to_string()).data(serde_json::to_string(event).unwrap_or_default())
}

/// Wraps a channel of frames into an SSE response with heartbeats.
pub fn response(rx: mpsc::Receiver<Event>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = futures::stream::unfold(rx, |mut rx| async move { rx.recv().await.map(|e| (Ok(e), rx)) });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(HEARTBEAT).text("heartbeat"))
}
