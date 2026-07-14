//! Content domain for the web extension.
//!
//! Owns content ingestion, persistence, link analytics, and search. Layered
//! the standard way:
//!
//! - [`repository`] — every `sqlx` call lives here. Returns rich types
//!   (`Content`, `LinkPerformance`, `SearchResult`, etc.) defined in
//!   `systemprompt_web_shared::models`.
//! - [`services`] — business logic that composes repositories with file IO
//!   (ingestion from the filesystem, search ranking, link-generation,
//!   validation). Handlers and jobs call services, not the repository directly.
//! - [`api`] — handler-shaped helpers that adapt service output to the
//!   admin/bridge HTTP layer.

pub mod api;
pub mod repository;
pub mod services;
