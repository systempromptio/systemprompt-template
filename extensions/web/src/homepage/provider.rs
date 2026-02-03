use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt::database::Database;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use super::config::HomepageConfig;

pub struct HomepagePageDataProvider {
    config: Arc<HomepageConfig>,
}

impl HomepagePageDataProvider {
    #[must_use]
    pub fn new(config: Arc<HomepageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PageDataProvider for HomepagePageDataProvider {
    fn provider_id(&self) -> &'static str {
        "homepage"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["homepage".to_string()]
    }

    #[allow(clippy::cast_possible_truncation)]
    async fn provide_page_data(&self, ctx: &PageContext<'_>) -> Result<Value> {
        let mut config = (*self.config).clone();

        if let Some(ref mut playbooks) = config.playbooks {
            if let Some(db) = ctx.db_pool::<Arc<Database>>() {
                if let Some(pool) = db.pool() {
                    for category in &mut playbooks.categories {
                        let prefix = format!("{}-%", category.id);
                        let count_result = sqlx::query_scalar!(
                            r#"
                            SELECT COUNT(*) as "count!"
                            FROM markdown_content
                            WHERE source_id = 'playbooks' AND slug LIKE $1
                            "#,
                            prefix
                        )
                        .fetch_one(&*pool)
                        .await;

                        match count_result {
                            Ok(c) => {
                                tracing::debug!(category = %category.id, count = c, "Playbook category count");
                                category.count = Some(c as i32);
                            }
                            Err(e) => {
                                tracing::warn!(category = %category.id, error = %e, "Failed to fetch playbook count");
                            }
                        }
                    }
                }
            } else {
                tracing::debug!("No database pool available for playbook counts");
            }
        }

        let config_value = serde_json::to_value(&config)?;
        Ok(serde_json::json!({ "site": { "homepage": config_value } }))
    }

    fn priority(&self) -> u32 {
        50
    }
}
