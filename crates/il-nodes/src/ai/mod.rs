//! AI nodes (use model roles through `ctx.models()`, never model ids).
//!
//! Owned by task B07. Add one file per node and list it here.

pub mod agent;
pub mod classify;
mod common;
pub mod decide;
pub mod describe_image;
pub mod extract;
pub mod summarise;
pub mod transcribe;
pub mod translate;
pub mod write;
