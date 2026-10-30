//! ffmpeg media operations (child processes, never blocking the runtime).
//!
//! Owned by task M01. Add one file per node and list it here.

pub mod captions;
pub mod convert;
pub mod cut_clips;
mod ffmpeg;
pub(crate) mod join;

pub mod merge_av;
pub mod resize;
#[cfg(test)]
mod tests;
pub mod thumbnail;
