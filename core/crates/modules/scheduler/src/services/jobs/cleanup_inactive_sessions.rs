use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{repository::AnalyticsSessionRepository, AppContext};

pub async fn cleanup_inactive_sessions(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    logger
        .info("cleanup", "Starting inactive session cleanup job")
        .await
        .ok();

    let session_repo = AnalyticsSessionRepository::new(db_pool.clone());

    // Close sessions inactive for >1 hour
    let closed_sessions = session_repo.cleanup_inactive_sessions(1).await?;

    logger
        .info(
            "cleanup",
            &format!(
                "Closed {} inactive sessions (>1h inactivity)",
                closed_sessions
            ),
        )
        .await
        .ok();

    Ok(())
}
