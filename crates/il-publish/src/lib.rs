//! Publishing. Executes flows only through `FlowRunner`. Route groups self-register via `RouteFactory`.

pub mod guide;
pub mod keys;
pub mod mcp;
pub mod openai;
pub mod widget;

/// Referenced by `il-server` so the linker keeps this crate's self-registrations.
pub fn link() {}
