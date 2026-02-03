use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use systemprompt::database::Database;
use systemprompt::extension::prelude::{ContentDataContext, ContentDataProvider};

use crate::utils::playbook_categories::{
    BUILD_PREFIX, CLI_PREFIX, CONTENT_PREFIX, GUIDE_PREFIX, VALIDATION_PREFIX,
};

pub struct PlaybooksContentDataProvider;

impl PlaybooksContentDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    fn get_category_prefix(slug: &str) -> &'static str {
        if slug.starts_with(GUIDE_PREFIX) {
            GUIDE_PREFIX
        } else if slug.starts_with(CLI_PREFIX) {
            CLI_PREFIX
        } else if slug.starts_with(BUILD_PREFIX) {
            BUILD_PREFIX
        } else if slug.starts_with(CONTENT_PREFIX) {
            CONTENT_PREFIX
        } else if slug.starts_with(VALIDATION_PREFIX) {
            VALIDATION_PREFIX
        } else {
            ""
        }
    }
}

impl Default for PlaybooksContentDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContentDataProvider for PlaybooksContentDataProvider {
    fn provider_id(&self) -> &'static str {
        "playbooks-content-enricher"
    }

    fn applies_to_sources(&self) -> Vec<String> {
        vec!["playbooks".to_string()]
    }

    async fn enrich_content(
        &self,
        ctx: &ContentDataContext<'_>,
        item: &mut serde_json::Value,
    ) -> Result<()> {
        let db = ctx
            .db_pool::<Arc<Database>>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in context"))?;

        let pool = db
            .pool()
            .ok_or_else(|| anyhow::anyhow!("Database pool not initialized"))?;

        let content_id = ctx.content_id();

        let row = sqlx::query!(
            r#"
            SELECT
                slug,
                COALESCE(related_playbooks, '[]'::jsonb) as "related_playbooks!",
                COALESCE(related_code, '[]'::jsonb) as "related_code!"
            FROM markdown_content
            WHERE id = $1
            "#,
            content_id
        )
        .fetch_one(&*pool)
        .await?;

        if let Some(obj) = item.as_object_mut() {
            obj.insert("related_playbooks".to_string(), row.related_playbooks);
            obj.insert("related_code".to_string(), row.related_code);
        }

        let current_slug = row.slug.as_str();
        let category_prefix = Self::get_category_prefix(current_slug);

        if !category_prefix.is_empty() {
            let same_category =
                get_same_category_playbooks(&pool, category_prefix, current_slug).await?;

            if let Some(obj) = item.as_object_mut() {
                obj.insert("same_category_playbooks".to_string(), json!(same_category));
            }
        }

        Ok(())
    }
}

#[derive(Clone, serde::Serialize)]
pub struct SameCategoryPlaybook {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub url: String,
}

async fn get_same_category_playbooks(
    pool: &sqlx::PgPool,
    category_prefix: &str,
    current_slug: &str,
) -> Result<Vec<SameCategoryPlaybook>> {
    let pattern = format!("{category_prefix}%");

    let rows = sqlx::query!(
        r#"
        SELECT slug, title, description
        FROM markdown_content
        WHERE source_id = 'playbooks'
          AND slug LIKE $1
          AND slug != $2
        ORDER BY title
        LIMIT 5
        "#,
        pattern,
        current_slug
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| SameCategoryPlaybook {
            url: format!("/playbooks/{}", row.slug),
            slug: row.slug,
            title: row.title,
            description: row.description,
        })
        .collect())
}
