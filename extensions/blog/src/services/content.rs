//! Content service - business logic for content management.

use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{ContentId, SourceId};

use crate::error::BlogError;
use crate::models::{Content, CreateContentParams};
use crate::repository::ContentRepository;

/// Service for managing blog content.
#[derive(Debug, Clone)]
pub struct ContentService {
    repo: ContentRepository,
}

impl ContentService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ContentRepository::new(pool),
        }
    }

    /// Create new content.
    pub async fn create(&self, params: &CreateContentParams) -> Result<Content, BlogError> {
        self.repo.create(params).await.map_err(BlogError::from)
    }

    /// Get content by ID.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<Content>, BlogError> {
        let id = ContentId::new(id.to_string());
        self.repo.get_by_id(&id).await.map_err(BlogError::from)
    }

    /// Get content by slug.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, BlogError> {
        self.repo.get_by_slug(slug).await.map_err(BlogError::from)
    }

    /// Get content by source ID and slug.
    pub async fn get_by_source_and_slug(
        &self,
        source_id: &str,
        slug: &str,
    ) -> Result<Option<Content>, BlogError> {
        let source_id = SourceId::new(source_id.to_string());
        self.repo.get_by_source_and_slug(&source_id, slug).await.map_err(BlogError::from)
    }

    /// List content with pagination.
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, BlogError> {
        self.repo.list(limit, offset).await.map_err(BlogError::from)
    }

    /// List content by source.
    pub async fn list_by_source(&self, source_id: &str) -> Result<Vec<Content>, BlogError> {
        let source_id = SourceId::new(source_id.to_string());
        self.repo.list_by_source(&source_id).await.map_err(BlogError::from)
    }

    /// Update content.
    pub async fn update(
        &self,
        id: &str,
        title: &str,
        description: &str,
        body: &str,
        keywords: &str,
        image: Option<&str>,
        version_hash: &str,
    ) -> Result<Content, BlogError> {
        let id = ContentId::new(id.to_string());
        self.repo
            .update(&id, title, description, body, keywords, image, version_hash)
            .await
            .map_err(BlogError::from)
    }

    /// Delete content by ID.
    pub async fn delete(&self, id: &str) -> Result<(), BlogError> {
        let id = ContentId::new(id.to_string());
        self.repo.delete(&id).await.map_err(BlogError::from)
    }
}
