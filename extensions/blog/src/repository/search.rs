//! Search repository - database access layer for search queries.

use crate::models::SearchResult;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::identifiers::CategoryId;

/// Repository for search operations.
#[derive(Debug, Clone)]
pub struct SearchRepository {
    pool: Arc<PgPool>,
}

impl SearchRepository {
    /// Create a new search repository.
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Search by category.
    pub async fn search_by_category(
        &self,
        category_id: &CategoryId,
        limit: i64,
    ) -> Result<Vec<SearchResult>, sqlx::Error> {
        sqlx::query_as!(
            SearchResult,
            r#"
            SELECT c.id as "id: _", c.slug as "slug!", c.title as "title!",
                   c.description as "description!", c.image,
                   c.source_id as "source_id: _", c.category_id as "category_id: _",
                   COALESCE(m.total_views, 0) as "view_count!"
            FROM markdown_content c
            LEFT JOIN content_performance_metrics m ON c.id = m.content_id
            WHERE c.category_id = $1
            ORDER BY m.total_views DESC NULLS LAST
            LIMIT $2
            "#,
            category_id.as_str(),
            limit
        )
        .fetch_all(&*self.pool)
        .await
    }

    /// Search by keyword.
    pub async fn search_by_keyword(
        &self,
        keyword: &str,
        limit: i64,
    ) -> Result<Vec<SearchResult>, sqlx::Error> {
        let pattern = format!("%{}%", keyword);
        sqlx::query_as!(
            SearchResult,
            r#"
            SELECT c.id as "id: _", c.slug as "slug!", c.title as "title!",
                   c.description as "description!", c.image,
                   c.source_id as "source_id: _", c.category_id as "category_id: _",
                   COALESCE(m.total_views, 0) as "view_count!"
            FROM markdown_content c
            LEFT JOIN content_performance_metrics m ON c.id = m.content_id
            WHERE (c.title ILIKE $1 OR c.description ILIKE $1 OR c.body ILIKE $1)
            ORDER BY m.total_views DESC NULLS LAST
            LIMIT $2
            "#,
            pattern,
            limit
        )
        .fetch_all(&*self.pool)
        .await
    }
}
