use crate::models::BlogMetrics;
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;

#[derive(Debug)]
pub struct MetricsRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl MetricsRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    pub async fn create_metrics(&self, metrics: &BlogMetrics) -> Result<()> {
        let query = include_str!("../../src/queries/core/metrics/create_metrics.sql");

        self.db
            .execute(
                &query,
                &[
                    &metrics.id,
                    &metrics.content_id,
                    &i64::from(metrics.total_views),
                    &i64::from(metrics.unique_visitors),
                    &i64::from(metrics.avg_time_on_page_seconds),
                    &i64::from(metrics.shares_total),
                    &i64::from(metrics.shares_linkedin),
                    &i64::from(metrics.shares_twitter),
                    &i64::from(metrics.comments_count),
                    &i64::from(metrics.search_impressions),
                    &i64::from(metrics.search_clicks),
                    &metrics.avg_search_position,
                    &i64::from(metrics.views_last_7_days),
                    &i64::from(metrics.views_last_30_days),
                    &metrics.trend_direction,
                ],
            )
            .await
            .context(format!(
                "Failed to create metrics for content: {}",
                metrics.content_id
            ))?;

        Ok(())
    }

    pub async fn get_metrics(&self, content_id: &str) -> Result<Option<BlogMetrics>> {
        let query = include_str!("../../src/queries/core/metrics/get_metrics.sql");

        let row = self
            .db
            .fetch_optional(&query, &[&content_id])
            .await
            .context(format!("Failed to get metrics for content: {content_id}"))?;

        row.map(|r| BlogMetrics::from_json_row(&r)).transpose()
    }

    pub async fn update_metrics(&self, metrics: &BlogMetrics) -> Result<()> {
        let query = include_str!("../../src/queries/core/metrics/update_metrics.sql");

        self.db
            .execute(
                &query,
                &[
                    &i64::from(metrics.total_views),
                    &i64::from(metrics.unique_visitors),
                    &i64::from(metrics.avg_time_on_page_seconds),
                    &i64::from(metrics.shares_total),
                    &i64::from(metrics.shares_linkedin),
                    &i64::from(metrics.shares_twitter),
                    &i64::from(metrics.comments_count),
                    &i64::from(metrics.search_impressions),
                    &i64::from(metrics.search_clicks),
                    &metrics.avg_search_position,
                    &i64::from(metrics.views_last_7_days),
                    &i64::from(metrics.views_last_30_days),
                    &metrics.trend_direction,
                    &metrics.content_id,
                ],
            )
            .await
            .context(format!(
                "Failed to update metrics for content: {}",
                metrics.content_id
            ))?;

        Ok(())
    }

    pub async fn upsert_metrics(&self, metrics: &BlogMetrics) -> Result<()> {
        let query = include_str!("../../src/queries/core/metrics/upsert_metrics.sql");

        self.db
            .execute(
                &query,
                &[
                    &metrics.id,
                    &metrics.content_id,
                    &i64::from(metrics.total_views),
                    &i64::from(metrics.unique_visitors),
                    &i64::from(metrics.avg_time_on_page_seconds),
                    &i64::from(metrics.shares_total),
                    &i64::from(metrics.shares_linkedin),
                    &i64::from(metrics.shares_twitter),
                    &i64::from(metrics.comments_count),
                    &i64::from(metrics.search_impressions),
                    &i64::from(metrics.search_clicks),
                    &metrics.avg_search_position,
                    &i64::from(metrics.views_last_7_days),
                    &i64::from(metrics.views_last_30_days),
                    &metrics.trend_direction,
                ],
            )
            .await
            .context(format!(
                "Failed to upsert metrics for content: {}",
                metrics.content_id
            ))?;

        Ok(())
    }

    pub async fn get_top_articles(&self, limit: i64) -> Result<Vec<BlogMetrics>> {
        let query = include_str!("../../src/queries/core/metrics/get_top_articles.sql");

        let rows = self
            .db
            .fetch_all(&query, &[&limit])
            .await
            .context(format!("Failed to get top articles with limit: {limit}"))?;

        rows.iter()
            .map(BlogMetrics::from_json_row)
            .collect::<Result<Vec<_>>>()
    }
}
