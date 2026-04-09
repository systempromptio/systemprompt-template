use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::admin::gamification;
use crate::error::MarketplaceError;

#[derive(Debug, Clone, Copy, Default)]
pub struct RecalculateGamificationJob;

#[async_trait::async_trait]
impl Job for RecalculateGamificationJob {
    fn name(&self) -> &'static str {
        "recalculate_gamification"
    }

    fn description(&self) -> &'static str {
        "Recalculates XP, ranks, streaks, and achievements for all users"
    }

    fn schedule(&self) -> &'static str {
        "0 */30 * * * *"
    }

    fn run_on_startup(&self) -> bool {
        true
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let start_time = std::time::Instant::now();

        tracing::info!("Recalculate gamification job started");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(MarketplaceError::Internal(
                "Database not available in job context".to_string(),
            ))?;

        let pool = db.write_pool().ok_or(MarketplaceError::Internal(
            "Write PgPool not available from database".to_string(),
        ))?;

        let updated = gamification::recalculate_all(&pool).await?;

        let duration_ms = u64::try_from(start_time.elapsed().as_millis()).unwrap_or(u64::MAX);

        tracing::info!(
            users_updated = updated,
            duration_ms,
            "Recalculate gamification job completed"
        );

        Ok(JobResult::success()
            .with_stats(updated, 0)
            .with_duration(duration_ms))
    }
}

systemprompt::traits::submit_job!(&RecalculateGamificationJob);
