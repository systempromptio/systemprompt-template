//! Governance webhook: the four-stage decision pipeline invoked on every tool
//! call.
//!
//! Scope check, secret scan, blocklist, then rate limit. Every decision is
//! audited with a trace id whether it allows or denies.

mod authz;
pub(crate) mod engine;
mod handler;
mod scope;
mod types;

pub(crate) use authz::govern_authz;
pub(crate) use engine::engine;
pub(crate) use handler::govern_tool_use;
