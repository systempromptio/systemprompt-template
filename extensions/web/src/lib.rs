pub mod admin;
pub mod api;
mod assets;
pub mod blog;
mod config_loader;
pub mod docs;
pub mod extenders;
pub mod extension;
mod extension_impl;
pub mod features;
pub mod homepage;
pub mod jobs;
pub mod navigation;
pub mod partials;
mod partials_animations;
pub mod repository;
mod schemas;
pub mod services;

// Re-exports from shared crate for API stability
pub use systemprompt_web_shared as shared;
pub use systemprompt_web_shared::html_escape;
pub use systemprompt_web_shared::{config, config_errors, error, models};
mod utils {
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
    FeaturesConfig,
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
    UpdateContentParams, UpdateContentParamsBuilder,
};
pub use services::{
    ContentService, IngestionService, LinkAnalyticsService, LinkGenerationService, LinkService,
    SearchService, ValidationService,
};
