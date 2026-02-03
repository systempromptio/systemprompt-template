use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use systemprompt::database::Database;
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};

use super::error::DocsError;

pub struct DocsContentDataProvider;

impl DocsContentDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for DocsContentDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentDataProvider for DocsContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "docs-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["documentation".to_string()]
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()> {
        let db = ctx
            .db_pool::<Arc<Database>>()
            .ok_or(DocsError::NoDatabaseInContext)?;

        let pool = db.pool().ok_or(DocsError::PoolNotInitialized)?;

        let content_id = ctx.content_id();

        let row = sqlx::query!(
            r#"
            SELECT
                slug,
                kind,
                source_id,
                COALESCE(after_reading_this, '[]'::jsonb) as "after_reading_this!",
                COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                COALESCE(related_code, '[]'::jsonb) as "related_code!"
            FROM markdown_content
            WHERE id = $1
            "#,
            content_id
        )
        .fetch_one(&*pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => DocsError::ContentNotFound(content_id.to_string()),
            other => DocsError::Database(other),
        })?;

        if let Some(obj) = item.as_object_mut() {
            obj.insert("after_reading_this".to_string(), row.after_reading_this);
            obj.insert("related_playbooks".to_string(), row.related_playbooks);
            obj.insert("related_code".to_string(), row.related_code);
        }

        let kind = row.kind.as_str();
        if kind == "docs-index" || kind == "docs-list" {
            let children = self.get_children(&pool, &row.source_id, &row.slug).await;
            if let Some(obj) = item.as_object_mut() {
                obj.insert("children".to_string(), json!(children));
            }
        }

        Ok(())
    }
}

#[derive(Clone, serde::Serialize)]
pub struct ChildDoc {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

impl DocsContentDataProvider {
    pub async fn get_children_static(
        &self,
        pool: &sqlx::PgPool,
        source_id: &str,
        current_slug: &str,
    ) -> Vec<ChildDoc> {
        self.get_children(pool, source_id, current_slug).await
    }

    async fn get_children(
        &self,
        pool: &sqlx::PgPool,
        source_id: &str,
        current_slug: &str,
    ) -> Vec<ChildDoc> {
        let is_root = current_slug.is_empty() || current_slug == "index";

        if is_root {
            match sqlx::query!(
                r#"
                SELECT slug, title, description
                FROM markdown_content
                WHERE source_id = $1
                  AND slug != ''
                  AND slug != 'index'
                  AND slug NOT LIKE '%/%'
                ORDER BY title
                "#,
                source_id
            )
            .fetch_all(pool)
            .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .map(|row| ChildDoc {
                        url: format!("/documentation/{}", row.slug),
                        slug: row.slug,
                        title: row.title,
                        description: row.description,
                    })
                    .collect(),
                Err(e) => {
                    tracing::error!(error = %e, source_id, "Failed to fetch root children docs");
                    Vec::new()
                }
            }
        } else {
            let slug_prefix = format!("{current_slug}%");
            let parent_depth = current_slug.matches('/').count();

            match sqlx::query!(
                r#"
                SELECT slug, title, description
                FROM markdown_content
                WHERE source_id = $1
                  AND slug LIKE $2
                  AND slug != $3
                ORDER BY title
                "#,
                source_id,
                slug_prefix,
                current_slug
            )
            .fetch_all(pool)
            .await
            {
                Ok(rows) => rows
                    .into_iter()
                    .filter(|row| row.slug.matches('/').count() == parent_depth + 1)
                    .map(|row| ChildDoc {
                        url: format!("/documentation/{}", row.slug),
                        slug: row.slug,
                        title: row.title,
                        description: row.description,
                    })
                    .collect(),
                Err(e) => {
                    tracing::error!(error = %e, source_id, current_slug, "Failed to fetch children docs");
                    Vec::new()
                }
            }
        }
    }
}
