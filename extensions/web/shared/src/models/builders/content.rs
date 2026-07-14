use crate::models::ContentKind;
use chrono::{DateTime, Utc};
use systemprompt::identifiers::{CategoryId, SourceId};

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
    pub category: Option<String>,
    pub source_id: SourceId,
    pub version_hash: String,
    pub links: serde_json::Value, // JSON: variable-shape template data
    pub after_reading_this: serde_json::Value, // JSON: variable-shape template data
    pub related_playbooks: serde_json::Value, // JSON: variable-shape template data
    pub related_code: serde_json::Value, // JSON: variable-shape template data
    pub related_docs: serde_json::Value, // JSON: variable-shape template data
}

/// Required fields for a new content row; optional fields are layered on via
/// the `CreateContentParams::with_*` builders.
#[derive(Debug, Clone)]
pub struct ContentSeed {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub published_at: DateTime<Utc>,
    pub source_id: SourceId,
}

impl CreateContentParams {
    #[must_use]
    pub fn new(seed: ContentSeed) -> Self {
        Self {
            slug: seed.slug,
            title: seed.title,
            description: seed.description,
            body: seed.body,
            author: seed.author,
            published_at: seed.published_at,
            keywords: String::new(),
            kind: ContentKind::default(),
            image: None,
            category_id: None,
            category: None,
            source_id: seed.source_id,
            version_hash: String::new(),
            links: serde_json::Value::Array(vec![]),
            after_reading_this: serde_json::Value::Array(vec![]),
            related_playbooks: serde_json::Value::Array(vec![]),
            related_code: serde_json::Value::Array(vec![]),
            related_docs: serde_json::Value::Array(vec![]),
        }
    }

    #[must_use]
    pub fn with_keywords(mut self, keywords: String) -> Self {
        self.keywords = keywords;
        self
    }

    #[must_use]
    pub const fn with_kind(mut self, kind: ContentKind) -> Self {
        self.kind = kind;
        self
    }

    #[must_use]
    pub fn with_image(mut self, image: Option<String>) -> Self {
        self.image = image;
        self
    }

    #[must_use]
    pub fn with_category_id(mut self, category_id: Option<CategoryId>) -> Self {
        self.category_id = category_id;
        self
    }

    #[must_use]
    pub fn with_category(mut self, category: Option<String>) -> Self {
        self.category = category;
        self
    }

    #[must_use]
    pub fn with_version_hash(mut self, version_hash: String) -> Self {
        self.version_hash = version_hash;
        self
    }

    #[must_use]
    pub fn with_links(mut self, links: serde_json::Value) -> Self {
        self.links = links;
        self
    }

    #[must_use]
    pub fn with_after_reading_this(mut self, after_reading_this: serde_json::Value) -> Self {
        self.after_reading_this = after_reading_this;
        self
    }

    #[must_use]
    pub fn with_related_playbooks(mut self, related_playbooks: serde_json::Value) -> Self {
        self.related_playbooks = related_playbooks;
        self
    }

    #[must_use]
    pub fn with_related_code(mut self, related_code: serde_json::Value) -> Self {
        self.related_code = related_code;
        self
    }

    #[must_use]
    pub fn with_related_docs(mut self, related_docs: serde_json::Value) -> Self {
        self.related_docs = related_docs;
        self
    }
}
