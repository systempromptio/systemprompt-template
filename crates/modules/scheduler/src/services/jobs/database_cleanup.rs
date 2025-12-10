use anyhow::Result;
use serde_json::json;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogLevel, LogService};
use systemprompt_core_system::repository::CleanupRepository;
use systemprompt_core_system::AppContext;

pub async fn database_cleanup(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    logger
        .info("scheduler", "Job started | job=database_cleanup")
        .await
        .ok();

    let pool = db_pool.pool_arc().expect("Database must be PostgreSQL");
    let cleanup_repo = CleanupRepository::new((*pool).clone());
    let mut total_deleted = 0u64;

    let orphaned_logs = cleanup_repo.delete_orphaned_logs().await?;
    total_deleted += orphaned_logs;

    let orphaned_mcp = cleanup_repo.delete_orphaned_mcp_executions().await?;
    total_deleted += orphaned_mcp;

    let old_logs = cleanup_repo.delete_old_logs(30).await?;
    total_deleted += old_logs;

    let oauth_codes = cleanup_repo.delete_expired_oauth_codes().await?;
    let oauth_tokens = cleanup_repo.delete_expired_oauth_tokens().await?;
    total_deleted += oauth_codes + oauth_tokens;

    logger
        .log(
            LogLevel::Info,
            "scheduler",
            &format!(
                "Job completed | job=database_cleanup, deleted={}",
                total_deleted
            ),
            Some(json!({
                "job_name": "database_cleanup",
                "total_deleted": total_deleted,
                "orphaned_logs": orphaned_logs,
                "orphaned_mcp": orphaned_mcp,
                "old_logs": old_logs,
                "oauth_codes": oauth_codes,
                "oauth_tokens": oauth_tokens,
                "duration_ms": start_time.elapsed().as_millis(),
            })),
        )
        .await
        .ok();

    Ok(())
}
