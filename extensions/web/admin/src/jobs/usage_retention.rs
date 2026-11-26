//! `plugin_usage_retention` job: expires raw evidence older than 90 days.
//!
//! One transaction: `expire_raw_evidence` (schema `32_raw_retention.sql`)
//! deletes the hook plane and what hangs off it, then each linked member's
//! expiry runs: the gateway request log with `gateway`, the marketplace
//! session bindings with `mcp`. The per-conversation record and the daily
//! rollups are kept, so every figure the console shows outlives the raw rows.

use crate::jobs::error::JobError;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

const RAW_RETENTION_DAYS: i64 = 90;

#[derive(Debug, Clone, Copy, Default)]
pub struct PluginUsageRetentionJob;

impl PluginUsageRetentionJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(RAW_RETENTION_DAYS);
        let mut tx = pool.begin().await?;
        let deleted =
            sqlx::query_scalar!(r#"SELECT expire_raw_evidence($1) AS "deleted!""#, cutoff)
                .fetch_one(&mut *tx)
                .await?;
        #[cfg(feature = "gateway")]
        sqlx::query_scalar!(r#"SELECT expire_ai_raw_evidence($1) AS "deleted!""#, cutoff)
            .fetch_one(&mut *tx)
            .await?;
        #[cfg(feature = "mcp")]
        sqlx::query_scalar!(
            r#"SELECT expire_marketplace_raw_evidence($1) AS "deleted!""#,
            cutoff
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
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
        vec![systemprompt_web_shared::JOB_TAG]
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
        let db = ctx.get::<DbPool>()?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
