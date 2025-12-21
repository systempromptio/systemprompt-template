//! BlogExtension - reference implementation of a full extension.

use std::sync::Arc;
use axum::Router;
use sqlx::PgPool;

use crate::{api, jobs::ContentIngestionJob, BlogConfig};

/// SQL schema definitions embedded at compile time.
pub const SCHEMA_MARKDOWN_CONTENT: &str = include_str!("../schema/001_markdown_content.sql");
pub const SCHEMA_MARKDOWN_CATEGORIES: &str = include_str!("../schema/002_markdown_categories.sql");
pub const SCHEMA_CAMPAIGN_LINKS: &str = include_str!("../schema/003_campaign_links.sql");
pub const SCHEMA_LINK_CLICKS: &str = include_str!("../schema/004_link_clicks.sql");
pub const SCHEMA_LINK_ANALYTICS_VIEWS: &str = include_str!("../schema/005_link_analytics_views.sql");
pub const SCHEMA_CONTENT_PERFORMANCE_METRICS: &str = include_str!("../schema/006_content_performance_metrics.sql");
pub const SCHEMA_MARKDOWN_FTS: &str = include_str!("../schema/007_markdown_fts.sql");

/// Blog extension providing content management, search, and analytics.
///
/// # Capabilities
///
/// - **Schema**: 7 database tables for content, links, and analytics
/// - **API**: REST endpoints for content CRUD, search, and link tracking
/// - **Jobs**: Hourly content ingestion from filesystem
///
/// # Dependencies
///
/// This extension has no dependencies on other extensions.
/// It only requires database and config capabilities from the context.
#[derive(Debug, Default, Clone)]
pub struct BlogExtension {
    #[allow(dead_code)]
    config: Option<BlogConfig>,
}

impl BlogExtension {
    /// Create a new blog extension with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new blog extension with the given configuration.
    pub fn with_config(config: BlogConfig) -> Self {
        Self {
            config: Some(config),
        }
    }

    /// Get the extension ID.
    pub const fn id() -> &'static str {
        "blog"
    }

    /// Get the extension name.
    pub const fn name() -> &'static str {
        "Blog & Content Management"
    }

    /// Get the extension version.
    pub const fn version() -> &'static str {
        "1.0.0"
    }

    /// Get the extension priority (higher runs first).
    pub const fn priority() -> u32 {
        100
    }

    /// Get all schema definitions in order.
    pub fn schemas() -> Vec<(&'static str, &'static str)> {
        vec![
            ("markdown_content", SCHEMA_MARKDOWN_CONTENT),
            ("markdown_categories", SCHEMA_MARKDOWN_CATEGORIES),
            ("campaign_links", SCHEMA_CAMPAIGN_LINKS),
            ("link_clicks", SCHEMA_LINK_CLICKS),
            ("link_analytics_views", SCHEMA_LINK_ANALYTICS_VIEWS),
            ("content_performance_metrics", SCHEMA_CONTENT_PERFORMANCE_METRICS),
            ("markdown_fts", SCHEMA_MARKDOWN_FTS),
        ]
    }

    /// Get the API router for this extension.
    pub fn router(&self, pool: Arc<PgPool>, config: BlogConfig) -> Router {
        api::router(pool, config)
    }

    /// Get the base path for API routes.
    pub const fn base_path() -> &'static str {
        "/api/v1/content"
    }

    /// Check if authentication is required for this extension's routes.
    pub const fn requires_auth() -> bool {
        false // Public content endpoints
    }

    /// Get the content ingestion job.
    pub fn ingestion_job() -> ContentIngestionJob {
        ContentIngestionJob
    }

    /// Get the redirect router (mounted separately at /r/).
    pub fn redirect_router(pool: Arc<PgPool>) -> Router {
        api::redirect_router(pool)
    }
}
