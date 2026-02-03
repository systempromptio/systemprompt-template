use crate::models::Content;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId};

#[derive(Debug, Clone)]
pub struct ContentQueryRepository {
    pool: Arc<PgPool>,
}

impl ContentQueryRepository {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            WHERE id = $1
            "#,
            id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn get_by_source_and_slug(
        &self,
        source_id: &SourceId,
        slug: &str,
    ) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            WHERE source_id = $1 AND slug = $2
            "#,
            source_id.as_str(),
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            ORDER BY published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            WHERE source_id = $1
            ORDER BY published_at DESC
            "#,
            source_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash,
                   COALESCE(links, '[]'::jsonb) as "links!",
                   COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                   COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                   COALESCE(related_code, '[]'::jsonb) as "related_code!",
                   COALESCE(related_docs, '[]'::jsonb) as "related_docs!",
                   updated_at
            FROM markdown_content
            ORDER BY published_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn get_slugs_by_source(
        &self,
        source_id: &SourceId,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar!(
            r#"SELECT slug FROM markdown_content WHERE source_id = $1"#,
            source_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await?;
        Ok(rows)
    }
}
