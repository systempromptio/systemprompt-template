pub mod content;
pub mod events;
pub mod ingestion;
pub mod link;
pub mod metrics;
pub mod search;
pub mod tag;

pub use content::{Content, ContentMetadata};
pub use events::{EngagementEventRequest, ShareEventRequest, ViewEvent, ViewEventRequest};
pub use ingestion::IngestionReport;
pub use link::{
    CampaignLink, CampaignPerformance, ContentJourneyNode, DestinationType, LinkClick,
    LinkPerformance, LinkType, UtmParams,
};
pub use metrics::{AnalyticsAggregate, BlogMetrics};
pub use search::{SearchFilters, SearchRequest, SearchResponse, SearchResult};
pub use tag::Tag;
