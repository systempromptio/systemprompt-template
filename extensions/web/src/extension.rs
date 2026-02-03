use std::sync::{Arc, OnceLock};

use crate::assets::web_assets;
use systemprompt::database::Database;
use systemprompt::extension::prelude::ContentDataProvider;
use systemprompt::extension::prelude::*;
use systemprompt::template_provider::{ComponentRenderer, PageDataProvider, PagePrerenderer};
use systemprompt::traits::Job;

use crate::blog::{BlogListPageDataProvider, BlogPostPageDataProvider};
use crate::config::BlogConfigValidated;
use crate::config_loader::{self, ConfigError};
use crate::docs::{DocsContentDataProvider, DocsPageDataProvider};
use crate::features::{FeaturePagePrerenderer, FeaturesConfig};
use crate::homepage::{HomepageConfig, HomepagePageDataProvider, HomepagePrerenderer};
use crate::navigation::{NavigationConfig, NavigationPageDataProvider};
use crate::partials::{
    AgenticMeshAnimationPartialRenderer, CliRemoteAnimationPartialRenderer, FooterPartialRenderer,
    HeadAssetsPartialRenderer, HeaderPartialRenderer, MemoryLoopAnimationPartialRenderer,
    RustMeshAnimationPartialRenderer, ScriptsPartialRenderer,
};
use crate::playbooks::{
    PlaybookPageDataProvider, PlaybooksContentDataProvider, PlaybooksListPageDataProvider,
};
use crate::{
    api,
    jobs::{
        ContentAnalyticsAggregationJob, ContentIngestionJob, ContentPrerenderJob,
        CopyExtensionAssetsJob, LlmsTxtGenerationJob, PublishPipelineJob, RobotsTxtGenerationJob,
        SitemapGenerationJob,
    },
};

static NAVIGATION_CONFIG: OnceLock<Result<Option<Arc<NavigationConfig>>, ConfigError>> =
    OnceLock::new();
static HOMEPAGE_CONFIG: OnceLock<Result<Option<Arc<HomepageConfig>>, ConfigError>> =
    OnceLock::new();
static FEATURES_CONFIG: OnceLock<Result<Option<Arc<FeaturesConfig>>, ConfigError>> =
    OnceLock::new();

pub const SCHEMA_MARKDOWN_CONTENT: &str = include_str!("../schema/001_markdown_content.sql");
pub const SCHEMA_MARKDOWN_CATEGORIES: &str = include_str!("../schema/002_markdown_categories.sql");
pub const SCHEMA_CAMPAIGN_LINKS: &str = include_str!("../schema/003_campaign_links.sql");
pub const SCHEMA_LINK_CLICKS: &str = include_str!("../schema/004_link_clicks.sql");
pub const SCHEMA_LINK_ANALYTICS_VIEWS: &str =
    include_str!("../schema/005_link_analytics_views.sql");
pub const SCHEMA_CONTENT_PERFORMANCE_METRICS: &str =
    include_str!("../schema/006_content_performance_metrics.sql");
pub const SCHEMA_MARKDOWN_FTS: &str = include_str!("../schema/007_markdown_fts.sql");
pub const SCHEMA_ENGAGEMENT_EVENTS: &str = include_str!("../schema/008_engagement_events.sql");
pub const SCHEMA_CONTENT_RELATED_METADATA: &str =
    include_str!("../schema/009_content_related_metadata.sql");
pub const SCHEMA_CONTENT_RELATED_DOCS: &str =
    include_str!("../schema/010_content_related_docs.sql");
pub const SCHEMA_CONTENT_CATEGORY: &str = include_str!("../schema/011_content_category.sql");

#[derive(Debug, Default, Clone)]
pub struct WebExtension {
    validated_config: Option<Arc<BlogConfigValidated>>,
}

impl WebExtension {
    pub const PREFIX: &'static str = "web";

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_validated_config(config: Arc<BlogConfigValidated>) -> Self {
        Self {
            validated_config: Some(config),
        }
    }

    #[must_use]
    pub fn validated_config(&self) -> Option<&Arc<BlogConfigValidated>> {
        self.validated_config.as_ref()
    }

    #[must_use]
    pub const fn base_path() -> &'static str {
        "/api/v1/links"
    }

    #[must_use]
    pub fn ingestion_job() -> ContentIngestionJob {
        ContentIngestionJob
    }

    #[must_use]
    pub fn navigation_config() -> Option<Arc<NavigationConfig>> {
        NAVIGATION_CONFIG
            .get_or_init(config_loader::load_navigation_config)
            .clone()
            .inspect_err(|e| tracing::error!(error = %e, "Navigation config error"))
            .ok()
            .flatten()
    }

    #[must_use]
    pub fn homepage_config() -> Option<Arc<HomepageConfig>> {
        HOMEPAGE_CONFIG
            .get_or_init(config_loader::load_homepage_config)
            .clone()
            .inspect_err(|e| tracing::error!(error = %e, "Homepage config error"))
            .ok()
            .flatten()
    }

    #[must_use]
    pub fn features_config() -> Option<Arc<FeaturesConfig>> {
        FEATURES_CONFIG
            .get_or_init(config_loader::load_features_config)
            .clone()
            .inspect_err(|e| tracing::error!(error = %e, "Features config error"))
            .ok()
            .flatten()
    }
}

impl Extension for WebExtension {
    fn metadata(&self) -> ExtensionMetadata {
        ExtensionMetadata {
            id: "web",
            name: "Web Content & Navigation",
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn page_data_providers(&self) -> Vec<Arc<dyn PageDataProvider>> {
        let mut providers: Vec<Arc<dyn PageDataProvider>> = vec![];

        if let Some(nav_config) = Self::navigation_config() {
            let branding = config_loader::load_branding_config()
                .inspect_err(|e| tracing::error!(error = %e, "Branding config error"))
                .ok()
                .flatten();
            providers.push(Arc::new(
                NavigationPageDataProvider::new(nav_config).with_branding(branding),
            ));
        }

        if let Some(homepage_config) = Self::homepage_config() {
            providers.push(Arc::new(HomepagePageDataProvider::new(homepage_config)));
        }

        providers.push(Arc::new(DocsPageDataProvider::new()));

        providers.push(Arc::new(BlogListPageDataProvider::new()));
        providers.push(Arc::new(BlogPostPageDataProvider::new()));
        providers.push(Arc::new(PlaybooksListPageDataProvider::new()));
        providers.push(Arc::new(PlaybookPageDataProvider::new()));

        providers
    }

    fn content_data_providers(&self) -> Vec<Arc<dyn ContentDataProvider>> {
        vec![
            Arc::new(DocsContentDataProvider::new()),
            Arc::new(PlaybooksContentDataProvider::new()),
        ]
    }

    fn page_prerenderers(&self) -> Vec<Arc<dyn PagePrerenderer>> {
        let mut prerenderers: Vec<Arc<dyn PagePrerenderer>> = vec![];

        if let Some(config) = Self::homepage_config() {
            prerenderers.push(Arc::new(HomepagePrerenderer::new(config)));
        }

        if let Some(config) = Self::features_config() {
            for page in &config.pages {
                prerenderers.push(Arc::new(FeaturePagePrerenderer::new(page.clone())));
            }
        }

        prerenderers
    }

    fn component_renderers(&self) -> Vec<Arc<dyn ComponentRenderer>> {
        vec![
            Arc::new(HeadAssetsPartialRenderer),
            Arc::new(HeaderPartialRenderer),
            Arc::new(FooterPartialRenderer),
            Arc::new(ScriptsPartialRenderer),
            Arc::new(CliRemoteAnimationPartialRenderer),
            Arc::new(RustMeshAnimationPartialRenderer),
            Arc::new(MemoryLoopAnimationPartialRenderer),
            Arc::new(AgenticMeshAnimationPartialRenderer),
        ]
    }

    fn schemas(&self) -> Vec<SchemaDefinition> {
        vec![
            SchemaDefinition::inline("markdown_content", SCHEMA_MARKDOWN_CONTENT),
            SchemaDefinition::inline("markdown_categories", SCHEMA_MARKDOWN_CATEGORIES),
            SchemaDefinition::inline("campaign_links", SCHEMA_CAMPAIGN_LINKS),
            SchemaDefinition::inline("link_clicks", SCHEMA_LINK_CLICKS),
            SchemaDefinition::inline("link_analytics_views", SCHEMA_LINK_ANALYTICS_VIEWS),
            SchemaDefinition::inline(
                "content_performance_metrics",
                SCHEMA_CONTENT_PERFORMANCE_METRICS,
            ),
            SchemaDefinition::inline("markdown_fts", SCHEMA_MARKDOWN_FTS),
            SchemaDefinition::inline("engagement_events", SCHEMA_ENGAGEMENT_EVENTS),
            SchemaDefinition::inline("content_related_metadata", SCHEMA_CONTENT_RELATED_METADATA),
            SchemaDefinition::inline("content_related_docs", SCHEMA_CONTENT_RELATED_DOCS),
            SchemaDefinition::inline("content_category", SCHEMA_CONTENT_CATEGORY),
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
        vec![
            Arc::new(ContentIngestionJob),
            Arc::new(CopyExtensionAssetsJob),
            Arc::new(ContentPrerenderJob),
            Arc::new(SitemapGenerationJob),
            Arc::new(LlmsTxtGenerationJob),
            Arc::new(RobotsTxtGenerationJob),
            Arc::new(PublishPipelineJob),
            Arc::new(ContentAnalyticsAggregationJob),
        ]
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

    fn declares_assets(&self) -> bool {
        true
    }

    fn required_assets(
        &self,
        paths: &dyn systemprompt::extension::AssetPaths,
    ) -> Vec<systemprompt::extension::AssetDefinition> {
        web_assets(paths)
    }
}

register_extension!(WebExtension);

pub type BlogExtension = WebExtension;
