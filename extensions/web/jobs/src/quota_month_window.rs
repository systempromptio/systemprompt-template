//! Daily rewrite of calendar-month quota windows in `ai_gateway_policies`.
//!
//! Core's quota buckets are fixed-length and epoch-aligned, so "this
//! calendar month" is expressed by the admin extension as a window whose
//! `window_seconds` is rewritten every day to run out at the month's end,
//! with yesterday's bucket carried into today's. The arithmetic and the
//! writes live in the admin crate
//! (`repositories::gateway_policies::{month_window, month_window_db}`);
//! this job is the schedule. It runs a few minutes after UTC midnight and
//! is idempotent for the rest of the day, so a manual run is always safe:
//! `systemprompt infra jobs run quota_month_window`.
//!
//! Interim until core grows calendar windows — see the admin module head
//! for the contract and its one documented gap (the first minute after
//! the rewrite, while core's policy cache is stale).

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};
use systemprompt_web_admin::repositories::gateway_policies::month_window_db::refresh_month_windows;

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct QuotaMonthWindowJob;

impl QuotaMonthWindowJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let report = refresh_month_windows(pool, chrono::Utc::now())
            .await
            .map_err(|e| JobError::other(e.to_string()))?;
        tracing::info!(
            policies_rewritten = report.policies_rewritten,
            windows_rewritten = report.windows_rewritten,
            buckets_carried = report.buckets_carried,
            buckets_purged = report.buckets_purged,
            "quota month windows refreshed"
        );
        let touched = u64::try_from(report.windows_rewritten).unwrap_or(u64::MAX);
        Ok(JobResult::success().with_stats(touched, 0))
    }
}

#[async_trait::async_trait]
impl Job for QuotaMonthWindowJob {
    fn name(&self) -> &'static str {
        "quota_month_window"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Rewrites calendar-month quota windows to run out at month end and carries \
         yesterday's usage into today's bucket"
    }

    // Why: five minutes past UTC midnight — after the day has turned, before
    // anyone's morning; a window still on yesterday's value until then is
    // one bucket that carries forward, not usage lost.
    fn schedule(&self) -> &'static str {
        "0 5 0 * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db.write_pool();
        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&QuotaMonthWindowJob);
