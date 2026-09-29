//! `plugin_usage_retention` job: expires raw evidence older than 90 days.
//!
//! One call to `expire_raw_evidence` (schema `32_raw_retention.sql`), which
//! deletes the hook plane, the gateway request log and what hangs off them in
//! one transaction. The per-conversation record and the daily rollups are
//! kept, so every figure the console shows outlives the raw rows.

use crate::error::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

const RAW_RETENTION_DAYS: i64 = 90;

#[derive(Debug, Clone, Copy, Default)]
pub struct PluginUsageRetentionJob;

impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(RAW_RETENTION_DAYS);
        let deleted =
            sqlx::query_scalar!(r#"SELECT expire_raw_evidence($1) AS "deleted!""#, cutoff)
                .fetch_one(pool)
                .await?;
        let deleted = u64::try_from(deleted).map_err(|error| JobError::other(error.to_string()))?;
        Ok(JobResult::success().with_stats(deleted, 0))
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
        "Expires hook events, gateway requests and their evidence older than 90 days"
    }
    fn schedule(&self) -> &'static str {
        "0 20 3 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        tracing::info!(actor = %ctx.actor().user_id, "Raw evidence retention invoked");
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
systemprompt::traits::submit_job!(&PluginUsageRetentionJob);
