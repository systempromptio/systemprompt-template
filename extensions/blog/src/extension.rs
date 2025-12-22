//! BlogExtension - Static Content Generator for blog/content management.
//!
//! # Architecture
//!
//! This extension integrates with the core Static Content Generator (SCG) pipeline:
//!
//! 1. **Content Ingestion**: Markdown files are parsed and stored in the database
//! 2. **Static Generation**: Core's `PublishContentJob` renders HTML via Handlebars templates
//! 3. **Static Serving**: Generated HTML files are served from `dist/blog/`
//!
//! Content is NOT served via API - it's pre-rendered to static HTML files.
//! The API only provides link tracking endpoints for analytics.

use std::sync::Arc;

use systemprompt::database::Database;
use systemprompt::extension::prelude::*;
use systemprompt::traits::Job;

use crate::{api, jobs::ContentIngestionJob};
use crate::config::BlogConfigValidated;

/// SQL schema definitions embedded at compile time.
pub const SCHEMA_MARKDOWN_CONTENT: &str = include_str!("../schema/001_markdown_content.sql");
pub const SCHEMA_MARKDOWN_CATEGORIES: &str = include_str!("../schema/002_markdown_categories.sql");
pub const SCHEMA_CAMPAIGN_LINKS: &str = include_str!("../schema/003_campaign_links.sql");
pub const SCHEMA_LINK_CLICKS: &str = include_str!("../schema/004_link_clicks.sql");
pub const SCHEMA_LINK_ANALYTICS_VIEWS: &str = include_str!("../schema/005_link_analytics_views.sql");
pub const SCHEMA_CONTENT_PERFORMANCE_METRICS: &str = include_str!("../schema/006_content_performance_metrics.sql");
pub const SCHEMA_MARKDOWN_FTS: &str = include_str!("../schema/007_markdown_fts.sql");

/// Blog extension providing static content generation and link analytics.
///
/// # Architecture
///
/// This is a **Static Content Generator (SCG)**, not an API-first service.
/// Content is pre-rendered to HTML files and served statically.
///
/// ## Content Flow
///
/// 1. Markdown files in `services/content/blog/` are ingested to database
/// 2. Core's `PublishContentJob` renders HTML using Handlebars templates
/// 3. Static HTML files are written to `dist/blog/{slug}/index.html`
/// 4. Files are served by a static file server (not this extension)
///
/// ## What This Extension Provides
///
/// - **Schema**: Database tables for content storage and analytics
/// - **Ingestion**: Job to parse markdown and store in database
/// - **Link Tracking**: API for generating and tracking links (analytics)
/// - **Redirect Router**: Handles `/r/{short_code}` for tracked clicks
///
/// ## What Core SCG Provides
///
/// - Template rendering (Handlebars)
/// - Static HTML generation
/// - Sitemap generation
/// - Image optimization
///
/// # Dependencies
///
/// This extension has no dependencies on other extensions.
/// It requires database access and integrates with core's generator.
#[derive(Debug, Default, Clone)]
pub struct BlogExtension {
    validated_config: Option<Arc<BlogConfigValidated>>,
}

impl BlogExtension {
    /// Extension config prefix for profile lookup.
    pub const PREFIX: &'static str = "blog";

    /// Create a new blog extension with default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new blog extension with validated configuration.
    ///
    /// The validated config is produced by `BlogExtension::validate()` at startup.
    pub fn with_validated_config(config: Arc<BlogConfigValidated>) -> Self {
        Self {
            validated_config: Some(config),
        }
    }

    /// Get the validated config if available.
    pub fn validated_config(&self) -> Option<&Arc<BlogConfigValidated>> {
        self.validated_config.as_ref()
    }

    /// Get the base path for link tracking API routes.
    ///
    /// Note: Blog content is served at `/blog/` as static HTML files,
    /// NOT via this API path.
    pub const fn base_path() -> &'static str {
        "/api/v1/links"
    }

    /// Get the content ingestion job.
    pub fn ingestion_job() -> ContentIngestionJob {
        ContentIngestionJob
    }
}

impl Extension for BlogExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "blog",
            name: "Blog & Content Management",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("markdown_content", SCHEMA_MARKDOWN_CONTENT),
            SchemaDefinition::inline("markdown_categories", SCHEMA_MARKDOWN_CATEGORIES),
            SchemaDefinition::inline("campaign_links", SCHEMA_CAMPAIGN_LINKS),
            SchemaDefinition::inline("link_clicks", SCHEMA_LINK_CLICKS),
            SchemaDefinition::inline("link_analytics_views", SCHEMA_LINK_ANALYTICS_VIEWS),
            SchemaDefinition::inline("content_performance_metrics", SCHEMA_CONTENT_PERFORMANCE_METRICS),
            SchemaDefinition::inline("markdown_fts", SCHEMA_MARKDOWN_FTS),
        ]
    }

    fn router(&self, ctx: &dyn ExtensionContext) -> Option<ExtensionRouter> {
        let db_handle = ctx.database();
        let db = db_handle.as_any().downcast_ref::<Database>()?;
        let pool = db.pool()?;

        let router = api::router(pool, self.validated_config.clone());

        Some(ExtensionRouter::new(router, Self::base_path()))
    }

    fn jobs(&self) -> Vec<Arc<dyn Job>> {
        vec![Arc::new(ContentIngestionJob)]
    }

    fn priority(&self) -> u32 {
        100
    }

    fn migration_weight(&self) -> u32 {
        100
    }

    fn config_prefix(&self) -> Option<&str> {
        Some(Self::PREFIX)
    }
}

register_extension!(BlogExtension);
