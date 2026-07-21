use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::analytics::content_rollup::{
    self, ContentRollupRow, UpsertMetricsParams,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentAnalyticsAggregationJob;

impl ContentAnalyticsAggregationJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();

        tracing::info!("Content analytics aggregation started");

        let stats = content_rollup::aggregate_engagement_stats(pool).await?;
        let total_count = stats.len();

        let (success_count, error_count) = Self::upsert_all_metrics(pool, stats).await;

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

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

    async fn upsert_all_metrics(pool: &PgPool, stats: Vec<ContentRollupRow>) -> (u64, u64) {
        let mut success_count = 0u64;
        let mut error_count = 0u64;

        for stat in stats {
            match Self::upsert_metrics(pool, &stat).await {
                Ok(()) => {
                    success_count += 1;
                    tracing::debug!(
                        content_id = %stat.content_id,
                        views = stat.total_views,
                        "Updated content metrics"
                    );
                },
                Err(e) => {
                    error_count += 1;
                    tracing::warn!(
                        content_id = %stat.content_id,
                        error = %e,
                        "Failed to update content metrics"
                    );
                },
            }
        }

        (success_count, error_count)
    }

    async fn upsert_metrics(pool: &PgPool, stats: &ContentRollupRow) -> Result<(), JobError> {
        let id = format!("cpm_{}", uuid::Uuid::new_v4());

        let previous_23d = stats.views_30d - stats.views_7d;
        let avg_previous_week = f64::from(i32::try_from(previous_23d).unwrap_or(0)) / 3.0;
        let views_7d_f64 = f64::from(i32::try_from(stats.views_7d).unwrap_or(0));
        let trend_direction = if views_7d_f64 > avg_previous_week * 1.2 {
            "up"
        } else if views_7d_f64 < avg_previous_week * 0.8 {
            "down"
        } else {
            "stable"
        };

        let params = UpsertMetricsParams {
            id: &id,
            content_id: stats.content_id.as_str(),
            total_views: i32::try_from(stats.total_views).unwrap_or(i32::MAX),
            unique_visitors: i32::try_from(stats.unique_visitors).unwrap_or(i32::MAX),
            avg_time_seconds: stats.avg_time_seconds,
            views_7d: i32::try_from(stats.views_7d).unwrap_or(i32::MAX),
            views_30d: i32::try_from(stats.views_30d).unwrap_or(i32::MAX),
            trend_direction,
        };

        content_rollup::upsert_metrics(pool, &params).await?;
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

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Content analytics aggregation invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;

        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&ContentAnalyticsAggregationJob);
