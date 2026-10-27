//! Knowledge. May use il-store for its own `kb_` tables; calls models only through `ModelGateway`.

pub mod chunk;
pub mod crawl;
pub mod ingest;
pub mod parse;
pub mod search;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
