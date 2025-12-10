use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::AppContext;

pub async fn cleanup_inactive_sessions(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=cleanup_inactive_sessions")
        .await
        .ok();

    let session_repo = AnalyticsSessionRepository::new(db_pool.clone());

    // Close sessions inactive for >1 hour
    let closed_sessions = session_repo.cleanup_inactive(1).await?;

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=cleanup_inactive_sessions, closed_sessions={}",
                closed_sessions
            ),
            Some(json!({
                "job_name": "cleanup_inactive_sessions",
                "processed": closed_sessions,
                "duration_ms": start_time.elapsed().as_millis(),
                "inactive_minutes": 60,
            })),
        )
        .await
        .ok();

    Ok(())
}
