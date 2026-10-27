//! Time source. Tests use `testkit::FakeClock`.

use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Abstract clock so schedules, waits and retries are testable.
#[async_trait]
pub trait Clock: Send + Sync {
    /// Current time.
    fn now(&self) -> DateTime<Utc>;
    /// Sleeps (fake clocks return when advanced past the deadline).
    async fn sleep(&self, duration: Duration);
}

/// The real clock.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

#[async_trait]
impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
