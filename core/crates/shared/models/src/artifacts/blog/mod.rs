use crate::artifacts::metadata::ExecutionMetadata;
use crate::artifacts::traits::Artifact;
use crate::artifacts::types::ArtifactType;
use crate::content::ContentLink;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

fn default_artifact_type() -> String {
    "blog".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlogArtifact {
    #[serde(rename = "x-artifact-type")]
    #[serde(default = "default_artifact_type")]
    pub artifact_type: String,
    pub title: String,
    pub slug: String,
    pub content: String,
    pub content_id: Option<String>,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub published_at: Option<String>,
    pub tags: Option<Vec<String>>,
    pub categories: Option<Vec<String>>,
    pub keywords: Option<String>,
    pub author: Option<String>,
    pub content_type: Option<String>,
    pub links: Option<Vec<ContentLink>>,
    #[serde(skip)]
    #[schemars(skip)]
    metadata: ExecutionMetadata,
}

impl BlogArtifact {
    pub fn new(
        title: impl Into<String>,
        slug: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            artifact_type: "blog".to_string(),
            title: title.into(),
            slug: slug.into(),
            content: content.into(),
            content_id: None,
            excerpt: None,
            featured_image_url: None,
            published_at: None,
            tags: None,
            categories: None,
            keywords: None,
            author: None,
            content_type: None,
            links: None,
            metadata: ExecutionMetadata::default(),
        }
    }

    pub fn with_content_id(mut self, id: impl Into<String>) -> Self {
        self.content_id = Some(id.into());
        self
    }

    pub fn with_excerpt(mut self, excerpt: impl Into<String>) -> Self {
        self.excerpt = Some(excerpt.into());
        self
    }

    pub fn with_featured_image(mut self, url: impl Into<String>) -> Self {
        self.featured_image_url = Some(url.into());
        self
    }

    pub fn with_published_at(mut self, datetime: impl Into<String>) -> Self {
        self.published_at = Some(datetime.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.categories = Some(categories);
        self
    }

    pub fn with_keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = Some(keywords.into());
        self
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn with_links(mut self, links: Vec<ContentLink>) -> Self {
        self.links = Some(links);
        self
    }

    pub fn with_execution_id(mut self, id: String) -> Self {
        self.metadata.execution_id = Some(id);
        self
    }

    pub fn with_skill(
        mut self,
        skill_id: impl Into<String>,
        skill_name: impl Into<String>,
    ) -> Self {
        self.metadata = self.metadata.with_skill(skill_id.into(), skill_name.into());
        self
    }
}

impl Artifact for BlogArtifact {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Blog
    }

    fn to_schema(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Blog post title"
                },
                "slug": {
                    "type": "string",
                    "description": "URL slug for the blog post"
                },
                "content": {
                    "type": "string",
                    "description": "Main body content (markdown)"
                },
                "content_id": {
                    "type": "string",
                    "description": "Unique identifier for the blog post"
                },
                "excerpt": {
                    "type": "string",
                    "description": "Optional brief summary"
                },
                "featured_image_url": {
                    "type": "string",
                    "description": "Optional featured image URL"
                },
                "published_at": {
                    "type": "string",
                    "description": "ISO datetime when published"
                },
                "tags": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional tags"
                },
                "categories": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Optional categories"
                },
                "keywords": {
                    "type": "string",
                    "description": "SEO keywords"
                },
                "author": {
                    "type": "string",
                    "description": "Content author"
                },
                "content_type": {
                    "type": "string",
                    "description": "Type of content (article, post, page, blog, documentation)"
                },
                "links": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "title": {"type": "string"},
                            "url": {"type": "string"}
                        },
                        "required": ["title", "url"]
                    },
                    "description": "Grounding references and sources"
                }
            },
            "required": ["title", "slug", "content"],
            "x-artifact-type": "blog"
        })
    }
}
