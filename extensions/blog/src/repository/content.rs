//! Content repository - database access layer.

use crate::models::{Content, CreateContentParams};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId};

/// Repository for content operations.
#[derive(Debug, Clone)]
pub struct ContentRepository {
    pool: Arc<PgPool>,
}

impl ContentRepository {
    /// Create a new content repository.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create a new content item.
    pub async fn create(&self, params: &CreateContentParams) -> Result<Content, sqlx::Error> {
        let id = ContentId::new(uuid::Uuid::new_v4().to_string());
        let now = Utc::now();
        sqlx::query_as!(
            Content,
            r#"
            INSERT INTO markdown_content (
                id, slug, title, description, body, author,
                published_at, keywords, kind, image, category_id, source_id,
                version_hash, links, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING id as "id: ContentId", slug, title, description, body, author,
                      published_at, keywords, kind, image,
                      category_id as "category_id: CategoryId",
                      source_id as "source_id: SourceId",
                      version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            "#,
            id.as_str(),
            params.slug,
            params.title,
            params.description,
            params.body,
            params.author,
            params.published_at,
            params.keywords,
            params.kind.as_str(),
            params.image,
            params.category_id.as_ref().map(CategoryId::as_str),
            params.source_id.as_str(),
            params.version_hash,
            params.links,
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    /// Get content by ID.
    pub async fn get_by_id(&self, id: &ContentId) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE id = $1
            "#,
            id.as_str()
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// Get content by slug.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE slug = $1
            "#,
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// Get content by source ID and slug.
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
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE source_id = $1 AND slug = $2
            "#,
            source_id.as_str(),
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    /// List content with pagination.
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
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

    /// List content by source.
    pub async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE source_id = $1
            ORDER BY published_at DESC
            "#,
            source_id.as_str()
        )
        .fetch_all(&*self.pool)
        .await
    }

    /// Update content.
    pub async fn update(
        &self,
        id: &ContentId,
        title: &str,
        description: &str,
        body: &str,
        keywords: &str,
        image: Option<&str>,
        version_hash: &str,
    ) -> Result<Content, sqlx::Error> {
        let now = Utc::now();
        sqlx::query_as!(
            Content,
            r#"
            UPDATE markdown_content
            SET title = $1, description = $2, body = $3, keywords = $4,
                image = $5, version_hash = $6, updated_at = $7
            WHERE id = $8
            RETURNING id as "id: ContentId", slug, title, description, body, author,
                      published_at, keywords, kind, image,
                      category_id as "category_id: CategoryId",
                      source_id as "source_id: SourceId",
                      version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            "#,
            title,
            description,
            body,
            keywords,
            image,
            version_hash,
            now,
            id.as_str()
        )
        .fetch_one(&*self.pool)
        .await
    }

    /// Delete content by ID.
    pub async fn delete(&self, id: &ContentId) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM markdown_content WHERE id = $1", id.as_str())
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    /// List all content with pagination.
    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id as "id: ContentId", slug, title, description, body, author,
                   published_at, keywords, kind, image,
                   category_id as "category_id: CategoryId",
                   source_id as "source_id: SourceId",
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
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
}
