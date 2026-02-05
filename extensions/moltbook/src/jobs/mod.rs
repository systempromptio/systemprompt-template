use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use systemprompt::traits::{Job, JobContext, JobResult};

#[derive(Debug, Clone, Copy, Default)]
pub struct MoltbookSyncJob;

#[async_trait]
impl Job for MoltbookSyncJob {
    fn name(&self) -> &'static str {
        "moltbook_sync"
    }

    fn description(&self) -> &'static str {
        "Synchronizes Moltbook agent activities and updates local analytics"
    }

    fn schedule(&self) -> &'static str {
        "0 0 */4 * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx
            .db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))?;

        tracing::info!("Running Moltbook sync job");

        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM moltbook_agents WHERE enabled = true")
                .fetch_one(pool)
                .await?;

        let count = row.0;

        tracing::info!(agent_count = count, "Moltbook sync complete");

        Ok(JobResult::success().with_message(format!("Synced {} Moltbook agents", count)))
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MoltbookAnalyticsJob;

#[derive(Debug, FromRow)]
struct AnalyticsStats {
    agent_count: i64,
    post_count: i64,
    total_upvotes: i64,
}

#[async_trait]
impl Job for MoltbookAnalyticsJob {
    fn name(&self) -> &'static str {
        "moltbook_analytics"
    }

    fn description(&self) -> &'static str {
        "Collects and aggregates Moltbook engagement analytics"
    }

    fn schedule(&self) -> &'static str {
        "0 30 * * * *"
    }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx
            .db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("Database pool not available"))?;

        tracing::info!("Running Moltbook analytics job");

        let stats: AnalyticsStats = sqlx::query_as(
            r"
            SELECT
                COUNT(DISTINCT agent_id) as agent_count,
                COUNT(*) as post_count,
                COALESCE(SUM(upvotes), 0) as total_upvotes
            FROM moltbook_posts
            WHERE created_at > NOW() - INTERVAL '24 hours'
            ",
        )
        .fetch_one(pool)
        .await?;

        tracing::info!(
            agents = stats.agent_count,
            posts = stats.post_count,
            upvotes = stats.total_upvotes,
            "Moltbook analytics collected"
        );

        Ok(JobResult::success().with_message(format!(
            "Analytics: {} agents, {} posts, {} upvotes (24h)",
            stats.agent_count, stats.post_count, stats.total_upvotes
        )))
    }
}
