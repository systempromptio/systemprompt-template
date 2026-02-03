use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use systemprompt::database::Database;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use crate::html_escape;
use crate::utils::playbook_categories::{
    BUILD, BUILD_PREFIX, CLI, CLI_PREFIX, CONTENT, CONTENT_PREFIX, GUIDE, GUIDE_PREFIX, VALIDATION,
    VALIDATION_PREFIX,
};

pub struct PlaybooksListPageDataProvider;

impl PlaybooksListPageDataProvider {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PlaybooksListPageDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Playbook {
    slug: String,
    title: String,
    description: String,
    published_at: DateTime<Utc>,
}

#[async_trait]
impl PageDataProvider for PlaybooksListPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "playbooks-list-posts"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["playbooks-list".to_string()]
    }

    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let db = ctx.db_pool::<Arc<Database>>().ok_or_else(|| {
            anyhow::anyhow!("PlaybooksListPageDataProvider: No database in context")
        })?;

        let pool = db.pool().ok_or_else(|| {
            anyhow::anyhow!("PlaybooksListPageDataProvider: Pool not initialized")
        })?;

        let playbooks = sqlx::query_as!(
            Playbook,
            r#"
            SELECT
                slug,
                title,
                description,
                published_at
            FROM markdown_content
            WHERE source_id = 'playbooks'
            ORDER BY slug ASC
            "#
        )
        .fetch_all(&*pool)
        .await?;

        tracing::debug!(count = playbooks.len(), "Fetched playbooks");

        let cards_html = render_playbook_cards(&playbooks);

        Ok(json!({
            "POSTS": cards_html
        }))
    }
}

fn derive_category(slug: &str) -> &str {
    if slug.starts_with(GUIDE_PREFIX) {
        GUIDE
    } else if slug.starts_with(CLI_PREFIX) {
        CLI
    } else if slug.starts_with(BUILD_PREFIX) {
        BUILD
    } else if slug.starts_with(CONTENT_PREFIX) {
        CONTENT
    } else if slug.starts_with(VALIDATION_PREFIX) {
        VALIDATION
    } else {
        ""
    }
}

fn render_playbook_cards(playbooks: &[Playbook]) -> String {
    playbooks
        .iter()
        .map(|playbook| {
            let category = derive_category(&playbook.slug);
            let date = playbook.published_at.format("%B %d, %Y").to_string();
            let date_iso = playbook.published_at.format("%Y-%m-%d").to_string();

            format!(
                r#"<a href="/playbooks/{slug}" class="content-card-link" data-category="{category}">
  <article class="content-card content-card--playbook content-card--category-{category}">
    <div class="card-content">
      <span class="card-category card-category--{category}">{category}</span>
      <h2 class="card-title">{title}</h2>
      <p class="card-description">{description}</p>
      <div class="meta">
        <time datetime="{date_iso}" class="meta-date">{date}</time>
      </div>
    </div>
  </article>
</a>"#,
                slug = playbook.slug,
                category = category,
                title = html_escape(&playbook.title),
                description = html_escape(&playbook.description),
                date = date,
                date_iso = date_iso,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
