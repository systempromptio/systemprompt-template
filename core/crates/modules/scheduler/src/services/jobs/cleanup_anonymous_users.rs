use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::AppContext;
use systemprompt_core_users::repository::UserRepository;

pub async fn cleanup_anonymous_users(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=cleanup_anonymous_users")
        .await
        .ok();

    let session_repo = AnalyticsSessionRepository::new(db_pool.clone());
    let expired_sessions = session_repo.cleanup_inactive(24).await?;

    let user_repo = UserRepository::new(db_pool.clone());
    let deleted_users = user_repo.cleanup_old_anonymous(30).await?;

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=cleanup_anonymous_users, expired_sessions={}, \
                 deleted_users={}",
                expired_sessions, deleted_users
            ),
            Some(json!({
                "job_name": "cleanup_anonymous_users",
                "expired_sessions": expired_sessions,
                "deleted_users": deleted_users,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}
