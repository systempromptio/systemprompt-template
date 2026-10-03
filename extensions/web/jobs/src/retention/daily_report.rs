//! The daily measurement: size, dead tuples and oldest row of every managed
//! table into `retention_runs`, with a warning when a table grew more than
//! a quarter week on week or an outbox is backing up.

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use super::all_managed;
use super::ledger::{measure, record_run, size_days_ago};
use crate::error::JobError;

// Why: growth above a quarter in a week is a change in behaviour, not
// traffic. The outbox is judged on age, not depth: the consumer drains
// within seconds when it is healthy, so anything pending for an hour is
// stalled — the production boot loop sat at 77k rows, under any depth
// threshold worth setting, with the oldest row two hours old.
const GROWTH_WARN_PERCENT: i64 = 25;
const OUTBOX_STALE_MINUTES: i64 = 60;

#[derive(Debug, Clone, Copy, Default)]
pub struct RetentionDailyReportJob;

impl RetentionDailyReportJob {
    pub(crate) async fn execute_with_pool(pool: &PgPool) -> Result<JobResult, JobError> {
        let started = std::time::Instant::now();
        let mut measured: u64 = 0;
        let mut warnings: Vec<String> = Vec::new();
        for table in all_managed() {
            let now = measure(pool, table).await?;
            record_run(pool, "daily", &now, table.window_days).await?;
            measured += 1;
            if let Some(before) = size_days_ago(pool, table.name, 7).await? {
                let grown = now.total_bytes.saturating_sub(before);
                if before > 0 && grown * 100 / before > GROWTH_WARN_PERCENT {
                    let line = format!(
                        "{} grew {} % in 7 days ({} -> {} bytes)",
                        table.name,
                        grown * 100 / before,
                        before,
                        now.total_bytes
                    );
                    tracing::warn!(table = table.name, "{line}");
                    warnings.push(line);
                }
            }
        }
        for (consumer, count, oldest_minutes) in pending_outbox(pool).await? {
            if oldest_minutes >= OUTBOX_STALE_MINUTES {
                let line = format!(
                    "event_outbox has {count} rows pending for {consumer}, oldest {oldest_minutes} minutes — the consumer is not draining"
                );
                tracing::warn!(consumer, count, oldest_minutes, "{line}");
                warnings.push(line);
            }
        }
        let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let message = if warnings.is_empty() {
            format!("{measured} tables measured")
        } else {
            format!("{measured} tables measured; {}", warnings.join("; "))
        };
        Ok(JobResult::success()
            .with_stats(measured, 0)
            .with_duration(duration_ms)
            .with_message(message))
    }
}

async fn pending_outbox(pool: &PgPool) -> Result<Vec<(String, i64, i64)>, JobError> {
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(consumer, '(relay)'), COUNT(*),
                COALESCE(EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - MIN(created_at))) / 60, 0)::bigint
         FROM event_outbox WHERE processed_at IS NULL GROUP BY consumer",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

#[async_trait::async_trait]
impl Job for RetentionDailyReportJob {
    fn name(&self) -> &'static str {
        "retention_daily_report"
    }
    fn tags(&self) -> Vec<&'static str> {
        vec![crate::registry::JOB_TAG]
    }
    fn description(&self) -> &'static str {
        "Records size, dead tuples and oldest row of every retention-managed table and warns on growth"
    }
    fn schedule(&self) -> &'static str {
        "0 30 4 * * *"
    }
    async fn execute(
        &self,
        ctx: &JobContext,
    ) -> Result<JobResult, systemprompt::traits::ProviderError> {
        let db = ctx.get::<DbPool>()?;
        Ok(Self::execute_with_pool(&db.write_pool()).await?)
    }
}
systemprompt::traits::submit_job!(&RetentionDailyReportJob);
