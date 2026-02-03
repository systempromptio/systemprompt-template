use std::sync::Arc;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::services::MemoryService;

#[derive(Debug, Clone, Copy, Default)]
pub struct MemorySynthesisJob;

impl MemorySynthesisJob {
    pub async fn execute_synthesis(pool: Arc<PgPool>) -> Result<JobResult> {
        let start = std::time::Instant::now();

        tracing::info!("Memory synthesis job started");

        let memory_service = MemoryService::new(pool);

        let expired_count = memory_service.cleanup_expired().await?;
        if expired_count > 0 {
            tracing::info!(expired_count, "Cleaned up expired memories");
        }

        let active_count = memory_service.count_active().await?;
        tracing::info!(active_count, "Active memories in database");

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            expired_cleaned = expired_count,
            active_memories = active_count,
            duration_ms,
            "Memory synthesis job completed"
        );

        #[allow(clippy::cast_sign_loss)]
        let count = active_count as u64;

        Ok(JobResult::success()
            .with_stats(count, 0)
            .with_duration(duration_ms))
    }
}

#[async_trait::async_trait]
impl Job for MemorySynthesisJob {
    fn name(&self) -> &'static str {
        "soul_memory_synthesis"
    }

    fn description(&self) -> &'static str {
        "Processes recent messages to extract and store memories. Also cleans up expired memories."
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

        Self::execute_synthesis(pool).await
    }
}

systemprompt::traits::submit_job!(&MemorySynthesisJob);
