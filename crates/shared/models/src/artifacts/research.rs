use crate::artifacts::card::PresentationCardResponse;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Research artifact containing both display card and structured data.
///
/// Used by research_blog to serialize and create_blog_post to deserialize.
/// This ensures compile-time type safety between producer and consumer.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchArtifact {
    /// Presentation card with research summary (for UI display)
    /// Flattened so card fields appear at top level for frontend compatibility
    #[serde(flatten)]
    pub card: PresentationCardResponse,

    /// Topic that was researched
    pub topic: String,

    /// List of source citations (structured JSON, not markdown)
    pub sources: Vec<SourceCitation>,

    /// Number of search queries used
    pub query_count: u32,

    /// Number of sources found
    pub source_count: u32,
}

impl ResearchArtifact {
    pub const ARTIFACT_TYPE: &'static str = "presentation_card";

    pub fn new(
        topic: impl Into<String>,
        card: PresentationCardResponse,
        sources: Vec<SourceCitation>,
    ) -> Self {
        let sources_len = sources.len() as u32;
        Self {
            card,
            topic: topic.into(),
            sources,
            query_count: 0,
            source_count: sources_len,
        }
    }

    pub const fn with_query_count(mut self, count: u32) -> Self {
        self.query_count = count;
        self
    }
}

/// Source citation with structured fields for programmatic access.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SourceCitation {
    /// Source title (often the domain name)
    pub title: String,

    /// Source URI
    pub uri: String,

    /// Relevance score (0.0 to 1.0)
    pub relevance: f32,
}

impl SourceCitation {
    pub fn new(title: impl Into<String>, uri: impl Into<String>, relevance: f32) -> Self {
        Self {
            title: title.into(),
            uri: uri.into(),
            relevance,
        }
    }
}
