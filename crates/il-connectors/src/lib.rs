//! App connectors. One folder per app; apps share state only via `NodeCtx`.

pub mod airtable;
pub mod bluesky;
pub mod discord;
pub mod gmail;
pub mod hubspot;
pub mod notion;
pub mod oauth;
pub mod openapi;
mod rest;
pub mod sheets;
pub mod shopify;
pub mod slack;
pub mod social_dry_run;
pub mod telegram;
pub mod youtube;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
