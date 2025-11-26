use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{repository::CleanupRepository, AppContext};

pub async fn database_cleanup(
    db_pool: DbPool,
    logger: LogService,
    _app_context: Arc<AppContext>,
) -> Result<()> {
    logger
        .info("db_cleanup", "Starting database cleanup job")
        .await
        .ok();

    let cleanup_repo = CleanupRepository::new(db_pool);
    let mut total_deleted = 0u64;

    logger
        .info("db_cleanup", "Step 1/4: Cleaning orphaned logs")
        .await
        .ok();

    let orphaned_logs = cleanup_repo.delete_orphaned_logs().await?;

    logger
        .info(
            "db_cleanup",
            &format!("Deleted {} orphaned log records", orphaned_logs),
        )
        .await
        .ok();
    total_deleted += orphaned_logs;

    logger
        .info("db_cleanup", "Step 2/4: Cleaning orphaned MCP executions")
        .await
        .ok();

    let orphaned_mcp = cleanup_repo.delete_orphaned_mcp_executions().await?;

    logger
        .info(
            "db_cleanup",
            &format!("Deleted {} orphaned MCP tool executions", orphaned_mcp),
        )
        .await
        .ok();
    total_deleted += orphaned_mcp;

    logger
        .info("db_cleanup", "Step 3/4: Cleaning old logs (>7 days)")
        .await
        .ok();

    let old_logs = cleanup_repo.delete_old_logs().await?;

    logger
        .info(
            "db_cleanup",
            &format!("Deleted {} old log records", old_logs),
        )
        .await
        .ok();
    total_deleted += old_logs;

    logger
        .info("db_cleanup", "Step 4/4: Cleaning expired OAuth data")
        .await
        .ok();

    let oauth_codes = cleanup_repo.delete_expired_oauth_codes().await?;
    let oauth_tokens = cleanup_repo.delete_expired_oauth_tokens().await?;

    logger
        .info(
            "db_cleanup",
            &format!(
                "Deleted {} OAuth codes and {} refresh tokens",
                oauth_codes, oauth_tokens
            ),
        )
        .await
        .ok();
    total_deleted += oauth_codes + oauth_tokens;

    logger
        .info(
            "db_cleanup",
            &format!(
                "Database cleanup complete. Total records deleted: {}",
                total_deleted
            ),
        )
        .await
        .ok();

    Ok(())
}
