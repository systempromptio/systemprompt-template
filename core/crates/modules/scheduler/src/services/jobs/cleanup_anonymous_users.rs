use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{repository::AnalyticsSessionRepository, AppContext};
use systemprompt_core_users::repository::UserRepository;

pub async fn cleanup_anonymous_users(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    logger
        .info("cleanup", "Starting anonymous user cleanup job")
        .await
        .ok();

    let session_repo = AnalyticsSessionRepository::new(db_pool.clone());
    let expired_sessions = session_repo.cleanup_expired_anonymous_sessions().await?;

    logger
        .info(
            "cleanup",
            &format!("Cleaned up {} expired anonymous sessions", expired_sessions),
        )
        .await
        .ok();

    let user_repo = UserRepository::new(db_pool.clone());
    let deleted_users = user_repo.cleanup_old_anonymous_users().await?;

    logger
        .info(
            "cleanup",
            &format!("Cleaned up {} old anonymous users", deleted_users),
        )
        .await
        .ok();

    logger
        .info(
            "cleanup",
            &format!(
                "Cleanup complete. Sessions: {}, Users: {}",
                expired_sessions, deleted_users
            ),
        )
        .await
        .ok();

    Ok(())
}
