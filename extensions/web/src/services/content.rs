use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{ContentId, SourceId};

use crate::error::BlogError;
use crate::models::{Content, CreateContentParams};
use crate::repository::{ContentRepository, UpdateContentParams};

#[derive(Debug, Clone)]
pub struct ContentService {
    repo: ContentRepository,
}

impl ContentService {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ContentRepository::new(pool),
        }
    }

    pub async fn create(&self, params: &CreateContentParams) -> Result<Content, BlogError> {
        self.repo.create(params).await.map_err(BlogError::from)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Content>, BlogError> {
        let id = ContentId::new(id.to_string());
        self.repo.get_by_id(&id).await.map_err(BlogError::from)
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, BlogError> {
        self.repo.get_by_slug(slug).await.map_err(BlogError::from)
    }

    pub async fn get_by_source_and_slug(
        &self,
        source_id: &str,
        slug: &str,
    ) -> Result<Option<Content>, BlogError> {
        let source_id = SourceId::new(source_id.to_string());
        self.repo
            .get_by_source_and_slug(&source_id, slug)
            .await
            .map_err(BlogError::from)
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, BlogError> {
        self.repo.list(limit, offset).await.map_err(BlogError::from)
    }

    pub async fn list_by_source(&self, source_id: &str) -> Result<Vec<Content>, BlogError> {
        let source_id = SourceId::new(source_id.to_string());
        self.repo
            .list_by_source(&source_id)
            .await
            .map_err(BlogError::from)
    }

    pub async fn update(&self, params: &UpdateContentParams) -> Result<Content, BlogError> {
        self.repo.update(params).await.map_err(BlogError::from)
    }

    pub async fn delete(&self, id: &str) -> Result<(), BlogError> {
        let id = ContentId::new(id.to_string());
        self.repo.delete(&id).await.map_err(BlogError::from)
    }
}
