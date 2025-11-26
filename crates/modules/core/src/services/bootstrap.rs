use crate::Config;
use anyhow::Result;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;
use systemprompt_core_logging::LogService;

pub async fn initialize_database() -> Result<()> {
    let app_context = crate::AppContext::new().await?;
    let log = LogService::system(app_context.db_pool().clone());
    let config = Config::from_env()?;
    let db_path = &config.database_url;

    let parent_dir = Path::new(db_path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid database path: {db_path}"))?;

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let db_url = format!("sqlite://{db_path}?mode=rwc");

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await?;

    sqlx::query("PRAGMA synchronous = NORMAL")
        .execute(&pool)
        .await?;

    let _ = log
        .info(
            "bootstrap",
            &format!("Database initialized at path: {db_path}"),
        )
        .await;

    pool.close().await;

    Ok(())
}
