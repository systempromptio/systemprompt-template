use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId, TagId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ContentKind {
    #[default]
    Blog,
    Guide,
    Tutorial,
    Reference,
    #[serde(rename = "docs-index")]
    DocsIndex,
    Docs,
    #[serde(rename = "docs-list")]
    DocsList,
    Feature,
    Playbook,
    Legal,
}

impl ContentKind {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Blog => "blog",
            Self::Guide => "guide",
            Self::Tutorial => "tutorial",
            Self::Reference => "reference",
            Self::DocsIndex => "docs-index",
            Self::Docs => "docs",
            Self::DocsList => "docs-list",
            Self::Feature => "feature",
            Self::Playbook => "playbook",
            Self::Legal => "legal",
        }
    }
}

impl std::fmt::Display for ContentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ContentKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "blog" => Ok(Self::Blog),
            "guide" => Ok(Self::Guide),
            "tutorial" => Ok(Self::Tutorial),
            "reference" => Ok(Self::Reference),
            "docs-index" => Ok(Self::DocsIndex),
            "docs" => Ok(Self::Docs),
            "docs-list" => Ok(Self::DocsList),
            "feature" => Ok(Self::Feature),
            "playbook" => Ok(Self::Playbook),
            "legal" | "page" => Ok(Self::Legal),
            _ => Err(format!("Unknown content kind: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Content {
    pub id: ContentId,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub published_at: DateTime<Utc>,
    pub keywords: String,
    pub kind: String,
    pub image: Option<String>,
    pub category_id: Option<CategoryId>,
    pub source_id: SourceId,
    pub version_hash: String,
    #[serde(default)]
    pub links: JsonValue,
    #[serde(default)]
    pub after_reading_this: JsonValue,
    #[serde(default)]
    pub related_playbooks: JsonValue,
    #[serde(default)]
    pub related_code: JsonValue,
    #[serde(default)]
    pub related_docs: JsonValue,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Content {
    pub const COLUMNS: &'static str = r#"
        id as "id: ContentId",
        slug,
        title,
        description,
        body,
        author,
        published_at,
        keywords,
        kind,
        image,
        category_id as "category_id: CategoryId",
        source_id as "source_id: SourceId",
        version_hash,
        COALESCE(links, '[]'::jsonb) as "links!",
        COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
        COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
        COALESCE(related_code, '[]'::jsonb) as "related_code!",
        COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
        updated_at
    "#;

    pub fn links_metadata(&self) -> Result<Vec<ContentLinkMetadata>, serde_json::Error> {
        serde_json::from_value(self.links.clone())
    }

    pub fn after_reading_this_list(&self) -> Result<Vec<String>, serde_json::Error> {
        serde_json::from_value(self.after_reading_this.clone())
    }

    pub fn related_playbooks_metadata(
        &self,
    ) -> Result<Vec<ContentLinkMetadata>, serde_json::Error> {
        serde_json::from_value(self.related_playbooks.clone())
    }

    pub fn related_code_metadata(&self) -> Result<Vec<ContentLinkMetadata>, serde_json::Error> {
        serde_json::from_value(self.related_code.clone())
    }

    pub fn related_docs_metadata(&self) -> Result<Vec<ContentLinkMetadata>, serde_json::Error> {
        serde_json::from_value(self.related_docs.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub published_at: String,
    pub slug: String,
    #[serde(default)]
    pub keywords: String,
    pub kind: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub links: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub after_reading_this: Vec<String>,
    #[serde(default)]
    pub related_playbooks: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub related_code: Vec<ContentLinkMetadata>,
    #[serde(default)]
    pub related_docs: Vec<ContentLinkMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLinkMetadata {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub slug: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionReport {
    pub files_found: usize,
    pub files_processed: usize,
    pub orphans_deleted: usize,
    pub errors: Vec<String>,
}

impl IngestionReport {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            files_found: 0,
            files_processed: 0,
            orphans_deleted: 0,
            errors: Vec::new(),
        }
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

impl Default for IngestionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IngestionOptions {
    pub override_existing: bool,
    pub recursive: bool,
    pub delete_orphans: bool,
}

impl IngestionOptions {
    #[must_use]
    pub const fn with_override(mut self, override_existing: bool) -> Self {
        self.override_existing = override_existing;
        self
    }

    #[must_use]
    pub const fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    #[must_use]
    pub const fn with_delete_orphans(mut self, delete_orphans: bool) -> Self {
        self.delete_orphans = delete_orphans;
        self
    }
}
