//! Blog Extension for SystemPrompt
//!
//! This is the reference implementation demonstrating how to build
//! a full-featured extension with:
//! - Database schemas
//! - API routes
//! - Background jobs
//! - Type-safe dependencies
//!
//! # Usage
//!
//! ```rust,ignore
//! use systemprompt_blog_extension::BlogExtension;
//!
//! // Register the extension with your application
//! let extension = BlogExtension::default();
//! ```

#![allow(clippy::module_name_repetitions)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::must_use_candidate)]

pub mod api;
pub mod config;
pub mod error;
pub mod extension;
pub mod jobs;
pub mod models;
pub mod repository;
pub mod services;

pub use config::{
    BlogConfigRaw, BlogConfigValidated, ContentSourceRaw, ContentSourceValidated,
    ExtensionConfigError, ExtensionConfigErrors,
};
pub use error::BlogError;
pub use extension::BlogExtension;

pub use models::{
    CampaignLink, CampaignPerformance, Content, ContentJourneyNode, ContentKind,
    ContentLinkMetadata, ContentMetadata, CreateContentParams, CreateLinkParams,
    DestinationType, IngestionOptions, IngestionReport, LinkClick, LinkPerformance,
    LinkType, PaperMetadata, PaperSection, RecordClickParams, SearchFilters, SearchRequest,
    SearchResponse, SearchResult, Tag, TrackClickParams, UtmParams,
};

pub use repository::{ContentRepository, LinkAnalyticsRepository, LinkRepository, SearchRepository};
pub use services::{
    ContentService, IngestionService, LinkAnalyticsService, LinkGenerationService, LinkService,
    SearchService, ValidationService,
};
pub use jobs::ContentIngestionJob;
