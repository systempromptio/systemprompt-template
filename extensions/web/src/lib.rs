pub mod api;
mod assets;
pub mod blog;
pub mod config;
mod config_errors;
mod config_loader;
pub mod docs;
pub mod error;
pub mod extension;
pub mod features;
pub mod homepage;
pub mod jobs;
pub mod models;
pub mod navigation;
pub mod partials;
pub mod playbooks;
pub mod repository;
pub mod services;
mod utils;

pub use utils::html_escape;

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
pub use navigation::{
    HeaderNavConfig, NavCta, NavItem, NavLink, NavSection, NavigationPageDataProvider,
};
pub use playbooks::{
    PlaybookPageDataProvider, PlaybooksContentDataProvider, PlaybooksListPageDataProvider,
    SameCategoryPlaybook,
};

pub use models::{
    CampaignLink, CampaignPerformance, Content, ContentJourneyNode, ContentKind,
    ContentLinkMetadata, ContentMetadata, CreateContentParams, CreateLinkParams, DestinationType,
    IngestionOptions, IngestionReport, LinkClick, LinkPerformance, LinkType, PaperMetadata,
    PaperSection, RecordClickParams, SearchFilters, SearchRequest, SearchResponse, SearchResult,
    Tag, TrackClickParams, UtmParams,
};

pub use jobs::ContentIngestionJob;
pub use repository::{
    ContentRepository, LinkAnalyticsRepository, LinkRepository, SearchRepository,
    UpdateContentParams, UpdateContentParamsBuilder,
};
pub use services::{
    ContentService, IngestionService, LinkAnalyticsService, LinkGenerationService, LinkService,
    SearchService, ValidationService,
};
