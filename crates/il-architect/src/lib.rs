//! The architect. Talks to models, runs and storage only through il-sdk traits.

pub mod appkit;
pub mod ask;
pub mod catalog;
pub mod compile;
pub mod layout;
pub mod testfix;
pub mod validate;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
