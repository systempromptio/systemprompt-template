//! Content creation parameters.

use crate::models::ContentKind;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::{CategoryId, SourceId};

/// Parameters for creating new content.
#[derive(Debug, Clone)]
pub struct CreateContentParams {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub published_at: DateTime<Utc>,
    pub keywords: String,
    pub kind: ContentKind,
    pub image: Option<String>,
    pub category_id: Option<CategoryId>,
    pub source_id: SourceId,
    pub version_hash: String,
    pub links: serde_json::Value,
}

impl CreateContentParams {
    /// Create new content parameters with required fields.
    pub fn new(
        slug: String,
        title: String,
        description: String,
        body: String,
        author: String,
        published_at: DateTime<Utc>,
        source_id: SourceId,
    ) -> Self {
        Self {
            slug,
            title,
            description,
            body,
            author,
            published_at,
            keywords: String::new(),
            kind: ContentKind::default(),
            image: None,
            category_id: None,
            source_id,
            version_hash: String::new(),
            links: serde_json::Value::Array(vec![]),
        }
    }

    /// Set keywords.
    pub fn with_keywords(mut self, keywords: String) -> Self {
        self.keywords = keywords;
        self
    }

    /// Set content kind.
    pub const fn with_kind(mut self, kind: ContentKind) -> Self {
        self.kind = kind;
        self
    }

    /// Set image URL.
    pub fn with_image(mut self, image: Option<String>) -> Self {
        self.image = image;
        self
    }

    /// Set category ID.
    pub fn with_category_id(mut self, category_id: Option<CategoryId>) -> Self {
        self.category_id = category_id;
        self
    }

    /// Set version hash.
    pub fn with_version_hash(mut self, version_hash: String) -> Self {
        self.version_hash = version_hash;
        self
    }

    /// Set links metadata.
    pub fn with_links(mut self, links: serde_json::Value) -> Self {
        self.links = links;
        self
    }
}
