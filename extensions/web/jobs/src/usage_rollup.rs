//! `usage_daily_rollup` job: recomputes `admin_usage_daily_rollups` for the
//! trailing window so the analytics dashboard reads one narrow table.
//!
//! Hourly rather than nightly so today's row stays usable on the dashboard;
//! the upserts are set-based and idempotent, so frequency is purely a
//! freshness choice.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::dashboard::usage_rollups;

// Why: yesterday and today — late-arriving events for the previous UTC day
// are still folded in on the next run after midnight.
const WINDOW_DAYS_BACK: i32 = 1;

#[derive(Debug, Clone, Copy, Default)]
pub struct UsageDailyRollupJob;

impl UsageDailyRollupJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let written =
            usage_rollups::upsert_daily_rollups_for_window(pool, WINDOW_DAYS_BACK).await?;
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(rows = written, duration_ms, "Usage daily rollup completed");
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for UsageDailyRollupJob {
    fn name(&self) -> &'static str {
        "usage_daily_rollup"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Rolls hook events, observed commits, and gateway requests up into per-user daily usage rows"
    }

    fn schedule(&self) -> &'static str {
        "0 5 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Usage daily rollup invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&UsageDailyRollupJob);
