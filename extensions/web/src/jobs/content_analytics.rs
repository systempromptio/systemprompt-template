use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug)]
struct ContentAnalyticsRow {
    content_id: String,
    total_views: i64,
    unique_visitors: i64,
    avg_time_seconds: f64,
    views_7d: i64,
    views_30d: i64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentAnalyticsAggregationJob;

impl ContentAnalyticsAggregationJob {
    pub async fn execute_with_pool(pool: Arc<PgPool>) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Content analytics aggregation started");

        let stats = Self::aggregate_engagement_stats(&pool).await?;

        let total_count = stats.len();
        let mut success_count = 0u64;
        let mut error_count = 0u64;

        for stat in stats {
            match Self::upsert_metrics(&pool, &stat).await {
                Ok(()) => {
                    success_count += 1;
                    tracing::debug!(
                        content_id = %stat.content_id,
                        views = stat.total_views,
                        "Updated content metrics"
                    );
                }
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(
                        content_id = %stat.content_id,
                        error = %e,
                        "Failed to update content metrics"
                    );
                }
            }
        }

        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            total = total_count,
            success = success_count,
            errors = error_count,
            duration_ms,
            "Content analytics aggregation completed"
        );

        Ok(JobResult::success()
            .with_stats(success_count, error_count)
            .with_duration(duration_ms))
    }

    async fn aggregate_engagement_stats(pool: &PgPool) -> Result<Vec<ContentAnalyticsRow>> {
        let rows = sqlx::query_as!(
            ContentAnalyticsRow,
            r#"
            SELECT
                mc.id as "content_id!",
                COUNT(*) FILTER (WHERE ee.time_on_page_ms > 0)::BIGINT as "total_views!",
                COUNT(DISTINCT ee.session_id)::BIGINT as "unique_visitors!",
                COALESCE(AVG(ee.time_on_page_ms)::DOUBLE PRECISION / 1000.0, 0) as "avg_time_seconds!",
                COUNT(*) FILTER (
                    WHERE ee.time_on_page_ms > 0
                    AND ee.created_at >= NOW() - INTERVAL '7 days'
                )::BIGINT as "views_7d!",
                COUNT(*) FILTER (
                    WHERE ee.time_on_page_ms > 0
                    AND ee.created_at >= NOW() - INTERVAL '30 days'
                )::BIGINT as "views_30d!"
            FROM engagement_events ee
            JOIN markdown_content mc ON (
                (ee.page_url LIKE '/blog/%' AND mc.slug = SUBSTRING(ee.page_url FROM 7) AND mc.source_id = 'blog')
                OR (ee.page_url LIKE '/documentation/%' AND mc.slug = SUBSTRING(ee.page_url FROM 16) AND mc.source_id = 'documentation')
                OR (ee.page_url LIKE '/playbooks/%' AND mc.slug = SUBSTRING(ee.page_url FROM 12) AND mc.source_id = 'playbooks')
                OR (ee.page_url LIKE '/legal/%' AND mc.slug = SUBSTRING(ee.page_url FROM 8) AND mc.source_id = 'legal')
            )
            GROUP BY mc.id
            HAVING COUNT(*) > 0
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(rows)
    }

    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    async fn upsert_metrics(pool: &PgPool, stats: &ContentAnalyticsRow) -> Result<()> {
        let id = format!("cpm_{}", uuid::Uuid::new_v4());

        let previous_23d = stats.views_30d - stats.views_7d;
        let avg_previous_week = previous_23d as f64 / 3.0; // ~3 weeks
        let trend_direction = if stats.views_7d as f64 > avg_previous_week * 1.2 {
            "up"
        } else if (stats.views_7d as f64) < avg_previous_week * 0.8 {
            "down"
        } else {
            "stable"
        };

        let total_views = stats.total_views as i32;
        let unique_visitors = stats.unique_visitors as i32;
        let views_7d = stats.views_7d as i32;
        let views_30d = stats.views_30d as i32;

        sqlx::query!(
            r#"
            INSERT INTO content_performance_metrics (
                id, content_id, total_views, unique_visitors,
                avg_time_on_page_seconds, views_last_7_days, views_last_30_days,
                trend_direction, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            ON CONFLICT (content_id) DO UPDATE SET
                total_views = EXCLUDED.total_views,
                unique_visitors = EXCLUDED.unique_visitors,
                avg_time_on_page_seconds = EXCLUDED.avg_time_on_page_seconds,
                views_last_7_days = EXCLUDED.views_last_7_days,
                views_last_30_days = EXCLUDED.views_last_30_days,
                trend_direction = EXCLUDED.trend_direction,
                updated_at = NOW()
            "#,
            id,
            stats.content_id,
            total_views,
            unique_visitors,
            stats.avg_time_seconds,
            views_7d,
            views_30d,
            trend_direction
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Job for ContentAnalyticsAggregationJob {
    fn name(&self) -> &'static str {
        "content_analytics_aggregation"
    }

    fn description(&self) -> &'static str {
        "Aggregates engagement events into content performance metrics (views, unique visitors, time on page)"
    }

    fn schedule(&self) -> &'static str {
        "0 */15 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or_else(|| anyhow::anyhow!("Database not available in job context"))?;

        let pool = db
            .pool()
            .ok_or_else(|| anyhow::anyhow!("PgPool not available from database"))?;

        Self::execute_with_pool(pool).await
    }
}

systemprompt::traits::submit_job!(&ContentAnalyticsAggregationJob);
