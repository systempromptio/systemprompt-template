use super::policies::{RetentionConfig, RetentionPolicy};
use crate::repository::LoggingRepository;
use chrono::Utc;
use systemprompt_core_database::DbPool;
use tokio_cron_scheduler::{Job, JobScheduler};

#[derive(Debug)]
pub struct RetentionScheduler {
    config: RetentionConfig,
    db_pool: DbPool,
}

impl RetentionScheduler {
    #[must_use]
    pub const fn new(config: RetentionConfig, db_pool: DbPool) -> Self {
        Self { config, db_pool }
    }

    pub async fn start(self) -> anyhow::Result<()> {
        if !self.config.enabled {
            tracing::info!("Log retention scheduler is disabled");
            return Ok(());
        }

        let schedule = &self.config.schedule;
        tracing::info!("Starting log retention scheduler with schedule: {schedule}");

        let scheduler = JobScheduler::new().await?;
        let config = self.config.clone();
        let db_pool = self.db_pool.clone();
        let schedule = config.schedule.clone();

        let job = Job::new_async(schedule.as_str(), move |_uuid, _lock| {
            let config = config.clone();
            let db_pool = db_pool.clone();

            Box::pin(async move {
                if let Err(e) = execute_retention_cleanup(config, db_pool).await {
                    tracing::error!("Retention cleanup failed: {e}");
                }
            })
        })?;

        scheduler.add(job).await?;
        scheduler.start().await?;

        tracing::info!("Log retention scheduler started successfully");

        Ok(())
    }
}

#[allow(clippy::cognitive_complexity)]
async fn execute_retention_cleanup(config: RetentionConfig, db_pool: DbPool) -> anyhow::Result<()> {
    tracing::info!("Starting scheduled log retention cleanup");

    let repo = LoggingRepository::new(db_pool.clone())
        .with_database(true)
        .with_terminal(false);

    let mut total_deleted = 0u64;

    for policy in &config.policies {
        let cutoff = Utc::now() - chrono::Duration::days(i64::from(policy.retention_days));

        match cleanup_by_policy(&repo, policy, cutoff).await {
            Ok(deleted) => {
                total_deleted += deleted;
                let policy_name = &policy.name;
                let retention_days = policy.retention_days;
                tracing::info!(
                    "Policy '{policy_name}' applied: deleted {deleted} logs (retention: {retention_days} days)"
                );
            },
            Err(e) => {
                let policy_name = &policy.name;
                tracing::error!("Failed to apply policy '{policy_name}': {e}");
            },
        }
    }

    tracing::info!("Retention cleanup completed - deleted {total_deleted} total logs");

    Ok(())
}

async fn cleanup_by_policy(
    repo: &LoggingRepository,
    _policy: &RetentionPolicy,
    cutoff: chrono::DateTime<Utc>,
) -> anyhow::Result<u64> {
    repo.cleanup_old_logs(cutoff).await.map_err(Into::into)
}
