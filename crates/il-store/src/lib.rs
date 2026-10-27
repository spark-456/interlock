//! Storage. No business logic. Tables are prefixed by module (see migrations/README.md).

pub mod db;
pub mod files;
pub mod repos;
pub mod secrets;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
