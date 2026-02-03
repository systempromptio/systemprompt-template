pub mod builders;
pub mod content;
pub mod engagement;
pub mod link;
pub mod paper;
pub mod search;

pub use builders::{CreateContentParams, CreateLinkParams, RecordClickParams, TrackClickParams};
pub use content::{
    Content, ContentKind, ContentLinkMetadata, ContentMetadata, IngestionOptions, IngestionReport,
    Tag,
};
pub use link::{
    CampaignLink, CampaignPerformance, ContentJourneyNode, DestinationType, LinkClick,
    LinkPerformance, LinkType, UtmParams,
};
pub use paper::{PaperMetadata, PaperSection};
pub use search::{SearchFilters, SearchRequest, SearchResponse, SearchResult};
