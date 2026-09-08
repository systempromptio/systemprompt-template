//! `scope_defaults_recompute` job: keeps each person's primary group and
//! project current so exclusive attribution partitions the instance.
//!
//! Membership handlers recompute inline when they change a container, so this
//! run exists for the changes nothing announces — a directory sign-in that
//! rewrites AD-sourced membership, or a container gaining members until it
//! outranks a person's previous primary.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;
use systemprompt_web_admin::repositories::scope::defaults;

#[derive(Debug, Clone, Copy, Default)]
pub struct ScopeDefaultsJob;

impl ScopeDefaultsJob {
    pub async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let written = defaults::recompute_scope_defaults(pool).await?;
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        tracing::info!(rows = written, duration_ms, "Scope defaults recomputed");
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for ScopeDefaultsJob {
    fn name(&self) -> &'static str {
        "scope_defaults_recompute"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Recomputes each user's primary group and project, leaving manual choices untouched"
    }

    fn schedule(&self) -> &'static str {
        "0 15 * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx
            .db_pool::<DbPool>()
            .ok_or(JobError::MissingContext("DbPool"))?;
        let pool = db
            .write_pool()
            .ok_or(JobError::MissingContext("write PgPool"))?;

        Ok(Self::execute_with_pool(&pool).await?)
    }
}

systemprompt::traits::submit_job!(&ScopeDefaultsJob);
