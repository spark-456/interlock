//! Repositories implementing `Store` and the store's additional records.

mod records;
mod store;

pub use records::{ApiKeyRecord, BuildRecord, ScheduleRecord, UserRecord, WebhookRecord, WorkspaceRecord};

#[cfg(test)]
mod tests;
