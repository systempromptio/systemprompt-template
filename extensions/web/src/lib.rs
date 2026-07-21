//! Web extension facade for the Enterprise Demo template.
//!
//! Aggregates the five web sibling crates (`admin`, `content`, `jobs`,
//! `shared`, `site`) into a single `WebExtension` registered with the core
//! runtime. Most of this crate is re-exports — the actual extension entry
//! point is [`extension::WebExtension`], which advertises required assets,
//! HTTP routes, page data providers, and jobs to the host binary.
//!
//! The split between siblings is the layering boundary:
//!
//! - `admin` — SSR dashboard, governance webhooks, bridge plane.
//! - `content` — content ingestion, repositories, search, link analytics.
//! - `jobs` — `publish_pipeline` and its sub-jobs (asset copy, prerender,
//!   sitemap, llms.txt, secret migration, content analytics).
//! - `shared` — config schemas, error types, branding, ID newtypes, HTML-escape
//!   helpers.
//! - `site` — public homepage / blog / docs / features content providers.

mod config_loader;

/// The admin template engine needs the same branding the server builds it
/// with; the HTTP contract suite would otherwise render every page against a
/// different context than production.
pub use config_loader::branding_config;
pub mod extension;
mod extension_impl;
mod schemas;

pub use systemprompt_web_admin as admin;
pub use systemprompt_web_content::{api, repository, services};
pub use systemprompt_web_jobs as jobs;
pub use systemprompt_web_shared as shared;
pub use systemprompt_web_shared::{
    BrandingConfig, config, config_errors, error, html_escape, models,
};
pub use systemprompt_web_site::{
    assets, blog, docs, extenders, features, homepage, navigation, partials,
};
pub mod utils {
    pub use systemprompt_web_shared::html_escape;
}

pub use blog::{BlogListPageDataProvider, BlogPostPageDataProvider};
pub use config::{
    BlogConfigRaw, BlogConfigValidated, ContentSourceRaw, ContentSourceValidated,
    ExtensionConfigError, ExtensionConfigErrors,
};
pub use docs::{ChildDoc, DocsContentDataProvider, DocsPageDataProvider};
pub use error::BlogError;
pub use extension::{BlogExtension, WebExtension};
pub use features::{
    FeatureCta, FeaturePage, FeaturePagePrerenderer, FeatureSection, FeatureSectionItem,
    FeaturePagesConfig,
};
pub use homepage::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DifferentiatorConfig, DifferentiatorItem,
    ExtensionTrait, ExtensionsConfig, FaqConfig, FaqItem, Feature, FeatureCategory, FinalCtaConfig,
    HeroConfig, HomepageConfig, HomepageFeaturesConfig, HomepagePageDataProvider, HowItWorksConfig,
    HowItWorksStep, IntegrationBrand, IntegrationsConfig, PricingConfig, PricingTier,
    TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use models::{
    CampaignLink, CampaignPerformance, Content, ContentJourneyNode, ContentKind,
    ContentLinkMetadata, ContentMetadata, CreateContentParams, CreateLinkParams, DestinationType,
    IngestionOptions, IngestionReport, LinkClick, LinkPerformance, LinkType, PaperMetadata,
    PaperSection, RecordClickParams, SearchFilters, SearchRequest, SearchResponse, SearchResult,
    Tag, TrackClickParams, UtmParams,
};
pub use navigation::{
    HeaderNavConfig, NavCta, NavItem, NavLink, NavSection, NavigationPageDataProvider,
};

pub use extenders::OrgUrlExtender;
pub use jobs::ContentIngestionJob;
pub use repository::{
    ContentRepository, LinkAnalyticsRepository, LinkRepository, SearchRepository,
    UpdateContentParams, UpdateContentParamsBuilder, UpdateContentSeed,
};
pub use services::{
    ContentService, IngestionService, LinkAnalyticsService, LinkGenerationService, LinkService,
    SearchService, ValidationService,
};
