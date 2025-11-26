use crate::artifacts::{metadata::ExecutionMetadata, traits::Artifact, types::ArtifactType};
use crate::content::ContentLink;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogArtifact {
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
    metadata: ExecutionMetadata,
}

impl BlogArtifact {
    pub fn new(
        title: impl Into<String>,
        slug: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self {
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

    pub fn to_response(&self) -> JsonValue {
        let mut response = json!({
            "x-artifact-type": "blog",
            "title": self.title,
            "slug": self.slug,
            "content": self.content
        });

        if let Some(ref id) = self.content_id {
            response["content_id"] = json!(id);
        }

        if let Some(ref excerpt) = self.excerpt {
            response["excerpt"] = json!(excerpt);
        }

        if let Some(ref url) = self.featured_image_url {
            response["featured_image_url"] = json!(url);
        }

        if let Some(ref datetime) = self.published_at {
            response["published_at"] = json!(datetime);
        }

        if let Some(ref tags) = self.tags {
            response["tags"] = json!(tags);
        }

        if let Some(ref categories) = self.categories {
            response["categories"] = json!(categories);
        }

        if let Some(ref keywords) = self.keywords {
            response["keywords"] = json!(keywords);
        }

        if let Some(ref author) = self.author {
            response["author"] = json!(author);
        }

        if let Some(ref content_type) = self.content_type {
            response["content_type"] = json!(content_type);
        }

        if let Some(ref links) = self.links {
            response["links"] = json!(links);
        }

        if let Some(ref id) = self.metadata.execution_id {
            response["_execution_id"] = json!(id);
        }

        response
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
