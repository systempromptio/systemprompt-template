//! Query mapping modules that map [`DatabaseQueryEnum`] variants to SQL file paths.
//!
//! Each module in this directory handles query mapping for one crate module.
//! For example, `agent.rs` maps all Agent module variants to their SQL files.
//!
//! The main delegation happens in `types.rs` which calls each module's
//! `get_query()` function in order until one returns `Some(&'static str)`.

pub mod agent;
pub mod ai;
pub mod blog;
pub mod config;
pub mod content_manager;
pub mod core;
pub mod log;
pub mod mcp;
pub mod oauth;
pub mod scheduler;
pub mod users;

// Re-export for types.rs
pub use agent::get_query as agent_get_query;
pub use ai::get_query as ai_get_query;
pub use blog::get_query as blog_get_query;
pub use config::get_query as config_get_query;
pub use content_manager::get_query as content_manager_get_query;
pub use core::get_query as core_get_query;
pub use log::get_query as log_get_query;
pub use mcp::get_query as mcp_get_query;
pub use oauth::get_query as oauth_get_query;
pub use scheduler::get_query as scheduler_get_query;
pub use users::get_query as users_get_query;
