//! `plugin_usage_retention` job: deletes `plugin_usage_events` rows older than
//! the retention window.
//!
//! Persisting `PreToolUse` (every *attempted* tool call, not just completed
//! ones) roughly doubles this table's write volume, and nothing else prunes
//! it. The daily rollups the dashboard reads are computed upstream of this
//! job and are never deleted, so trimming raw events costs history on the
//! trace explorer, not on the analytics tiles.
//!
//! Nightly and set-based; a run that deletes nothing is the steady state.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

// Why: 90 days covers a full quarter of trace lookback, which is the longest
// window the analytics UI can ask for (30d) plus room to investigate after the
// fact. Raise it and the table grows without bound; lower it and the trace
// explorer starts losing sessions a reader can still reach from a rollup.
const RETENTION_DAYS: i32 = 90;

#[derive(Debug, Clone, Copy, Default)]
pub struct PluginUsageRetentionJob;

impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let deleted = sqlx::query!(
            "DELETE FROM plugin_usage_events
             WHERE created_at < NOW() - make_interval(days => $1::INT)",
            RETENTION_DAYS,
        )
        .execute(pool)
        .await?
        .rows_affected();

        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(
            rows = deleted,
            retention_days = RETENTION_DAYS,
            duration_ms,
            "Plugin usage retention sweep completed"
        );
        Ok(JobResult::success()
            .with_stats(deleted, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for PluginUsageRetentionJob {
    fn name(&self) -> &'static str {
        "plugin_usage_retention"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Deletes plugin_usage_events rows past the retention window (rollups are unaffected)"
    }

    fn schedule(&self) -> &'static str {
        "0 20 3 * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id.as_str(), "Plugin usage retention invoked");

        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&PluginUsageRetentionJob);
