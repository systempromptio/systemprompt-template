//! `conversation_rollup` job: keeps `conversation_facts` current.
//!
//! Every minute it re-derives the fact row, and the per-skill rows beside it,
//! for every conversation touched on any plane — a request settled, a hook
//! event, a governance decision, a tool execution, an artifact — since the
//! stored watermark, through `refresh_conversation_facts_pending` (schema
//! 45_). Consecutive ticks cover disjoint windows, so a conversation is
//! rebuilt once per change rather than once per overlapping tick, and a
//! missed tick is caught up by the next. `-p all=true` rebuilds every row.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::error::JobError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ConversationRollupJob;

impl ConversationRollupJob {
    pub async fn execute_with_pool(pool: &PgPool, all: bool) -> Result<JobResult, JobError> {
        let start = std::time::Instant::now();
        let written = if all {
            sqlx::query_scalar!(r#"SELECT refresh_conversation_facts(NULL) AS "written!""#)
                .fetch_one(pool)
                .await?
        } else {
            sqlx::query_scalar!(r#"SELECT refresh_conversation_facts_pending() AS "written!""#)
                .fetch_one(pool)
                .await?
        };
        let duration_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
        let written = u64::try_from(written).unwrap_or(0);
        tracing::debug!(
            rows = written,
            duration_ms,
            all,
            "conversation rollup completed"
        );
        Ok(JobResult::success()
            .with_stats(written, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for ConversationRollupJob {
    fn name(&self) -> &'static str {
        "conversation_rollup"
    }

    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }

    fn description(&self) -> &'static str {
        "Re-derives the per-conversation fact rows for every conversation touched since the watermark"
    }

    fn schedule(&self) -> &'static str {
        "30 * * * * *"
    }

    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        let all = ctx.get_parameter_parsed::<bool>("all")?.unwrap_or(false);
        Ok(Self::execute_with_pool(&db.write_pool(), all).await?)
    }
}

systemprompt::traits::submit_job!(&ConversationRollupJob);
