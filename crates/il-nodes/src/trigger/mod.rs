//! Trigger nodes: manual, schedule, webhook, chat, API and MCP invocation.
//!
//! Owned by task B05. Add one file per node and list it here.

pub mod api_call;
pub mod chat;
pub mod manual;
pub mod mcp_call;
pub mod schedule;
pub mod webhook;
