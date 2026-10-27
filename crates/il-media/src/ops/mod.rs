//! ffmpeg media operations (child processes, never blocking the runtime).
//!
//! Owned by task M01. Add one file per node and list it here.

pub mod captions;
pub mod cut_clips;
pub mod merge_av;
pub mod resize;
pub mod thumbnail;
