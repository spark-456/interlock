//! The model gateway. Model ids live only in config; code asks for a role.

pub mod gateway;
pub mod providers;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
