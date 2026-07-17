use crate::models::{ContentKind, ContentLinkMetadata};
use chrono::{DateTime, Utc};
use sqlx::types::Json;
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
    pub links: Json<Vec<ContentLinkMetadata>>,
    pub after_reading_this: Json<Vec<String>>,
    pub related_playbooks: Json<Vec<ContentLinkMetadata>>,
    pub related_code: Json<Vec<ContentLinkMetadata>>,
    pub related_docs: Json<Vec<ContentLinkMetadata>>,
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
            links: Json(Vec::new()),
            after_reading_this: Json(Vec::new()),
            related_playbooks: Json(Vec::new()),
            related_code: Json(Vec::new()),
            related_docs: Json(Vec::new()),
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
    pub fn with_links(mut self, links: Vec<ContentLinkMetadata>) -> Self {
        self.links = Json(links);
        self
    }

    #[must_use]
    pub fn with_after_reading_this(mut self, after_reading_this: Vec<String>) -> Self {
        self.after_reading_this = Json(after_reading_this);
        self
    }

    #[must_use]
    pub fn with_related_playbooks(mut self, related_playbooks: Vec<ContentLinkMetadata>) -> Self {
        self.related_playbooks = Json(related_playbooks);
        self
    }

    #[must_use]
    pub fn with_related_code(mut self, related_code: Vec<ContentLinkMetadata>) -> Self {
        self.related_code = Json(related_code);
        self
    }

    #[must_use]
    pub fn with_related_docs(mut self, related_docs: Vec<ContentLinkMetadata>) -> Self {
        self.related_docs = Json(related_docs);
        self
    }
}
