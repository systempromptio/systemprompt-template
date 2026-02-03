mod mutations;
mod queries;

pub use mutations::{UpdateContentParams, UpdateContentParamsBuilder};

use crate::models::{Content, CreateContentParams};
use mutations::ContentMutationRepository;
use queries::ContentQueryRepository;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{ContentId, SourceId};

#[derive(Debug, Clone)]
pub struct ContentRepository {
    queries: ContentQueryRepository,
    mutations: ContentMutationRepository,
}

impl ContentRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            queries: ContentQueryRepository::new(Arc::clone(&pool)),
            mutations: ContentMutationRepository::new(pool),
        }
    }

    pub async fn create(&self, params: &CreateContentParams) -> Result<Content, sqlx::Error> {
        self.mutations.create(params).await
    }

    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        self.queries.get_by_id(id).await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        self.queries.get_by_slug(slug).await
    }

    pub async fn get_by_source_and_slug(
        &self,
        source_id: &SourceId,
        slug: &str,
    ) -> Result<Option<Content>, sqlx::Error> {
        self.queries.get_by_source_and_slug(source_id, slug).await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        self.queries.list(limit, offset).await
    }

    pub async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Content>, sqlx::Error> {
        self.queries.list_by_source(source_id).await
    }

    pub async fn update(&self, params: &UpdateContentParams) -> Result<Content, sqlx::Error> {
        self.mutations.update(params).await
    }

    pub async fn delete(&self, id: &ContentId) -> Result<(), sqlx::Error> {
        self.mutations.delete(id).await
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        self.queries.list_all(limit, offset).await
    }

    pub async fn get_slugs_by_source(
        &self,
        source_id: &SourceId,
    ) -> Result<Vec<String>, sqlx::Error> {
        self.queries.get_slugs_by_source(source_id).await
    }

    pub async fn delete_orphaned_slugs(
        &self,
        source_id: &SourceId,
        found_slugs: &[String],
    ) -> Result<u64, sqlx::Error> {
        self.mutations
            .delete_orphaned_slugs(source_id, found_slugs)
            .await
    }
}
