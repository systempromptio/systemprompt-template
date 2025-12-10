use crate::models::Content;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use uuid::Uuid;

#[derive(Debug)]
pub struct ContentRepository {
    pool: Arc<PgPool>,
}

impl ContentRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn create(
        &self,
        slug: &str,
        title: &str,
        description: &str,
        body: &str,
        author: &str,
        published_at: DateTime<Utc>,
        keywords: &str,
        kind: &str,
        image: Option<&str>,
        category_id: Option<&str>,
        source_id: &str,
        version_hash: &str,
        links: &serde_json::Value,
    ) -> Result<Content, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
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
            RETURNING id, slug, title, description, body, author,
                      published_at, keywords, kind, image, category_id, source_id,
                      version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            "#,
            id,
            slug,
            title,
            description,
            body,
            author,
            published_at,
            keywords,
            kind,
            image,
            category_id,
            source_id,
            version_hash,
            links,
            now
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
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
        source_id: &str,
        slug: &str,
    ) -> Result<Option<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE source_id = $1 AND slug = $2
            "#,
            source_id,
            slug
        )
        .fetch_optional(&*self.pool)
        .await
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
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

    pub async fn list_by_source(&self, source_id: &str) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
                   version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            FROM markdown_content
            WHERE source_id = $1
            ORDER BY published_at DESC
            "#,
            source_id
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn update(
        &self,
        id: &str,
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
            RETURNING id, slug, title, description, body, author,
                      published_at, keywords, kind, image, category_id, source_id,
                      version_hash, COALESCE(links, '[]'::jsonb) as "links!", updated_at
            "#,
            title,
            description,
            body,
            keywords,
            image,
            version_hash,
            now,
            id
        )
        .fetch_one(&*self.pool)
        .await
    }

    pub async fn delete(&self, id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM markdown_content WHERE id = $1", id)
            .execute(&*self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_by_source(&self, source_id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query!(
            "DELETE FROM markdown_content WHERE source_id = $1",
            source_id
        )
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn list_all(&self, limit: i64, offset: i64) -> Result<Vec<Content>, sqlx::Error> {
        sqlx::query_as!(
            Content,
            r#"
            SELECT id, slug, title, description, body, author,
                   published_at, keywords, kind, image, category_id, source_id,
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

    pub async fn get_popular_content_ids(
        &self,
        source_id: &str,
        days: i32,
        limit: i64,
    ) -> Result<Vec<String>, sqlx::Error> {
        let rows = sqlx::query_scalar!(
            r#"
            SELECT mc.id
            FROM markdown_content mc
            LEFT JOIN analytics_events ae ON
                ae.event_type = 'page_view'
                AND ae.event_category = 'content'
                AND ae.endpoint = 'GET /blog/' || mc.slug
                AND ae.timestamp >= CURRENT_TIMESTAMP - ($2 || ' days')::INTERVAL
            LEFT JOIN users u ON ae.user_id = u.id
            WHERE mc.source_id = $1
            GROUP BY mc.id, mc.published_at
            ORDER BY COUNT(DISTINCT CASE
                WHEN u.id IS NOT NULL AND u.is_bot = FALSE AND u.is_scanner = FALSE
                THEN ae.user_id
            END) DESC, mc.published_at DESC
            LIMIT $3
            "#,
            source_id,
            days.to_string(),
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows)
    }
}
