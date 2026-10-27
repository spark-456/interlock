//! Media operations (ffmpeg child processes) and `MediaGen` adapters.

pub mod r#gen;
pub mod ops;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
