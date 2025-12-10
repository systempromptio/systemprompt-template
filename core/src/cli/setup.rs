use anyhow::Result;
use clap::Args;
use sqlx::sqlite::SqlitePoolOptions;
use std::path::Path;
use std::sync::Arc;
use systemprompt_core_database::Database;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::models::modules::Modules;
use systemprompt_core_system::services::install::install_module;
use systemprompt_core_system::Config;

#[derive(Args)]
pub struct SetupArgs {
    /// Non-interactive mode
    #[arg(long)]
    non_interactive: bool,
}

async fn initialize_database(logger: &LogService) -> Result<()> {
    let config = Config::from_env()?;

    if config.database_type.eq_ignore_ascii_case("postgres")
        || config.database_type.eq_ignore_ascii_case("postgresql")
    {
        logger
            .info(
                "setup",
                &format!("Using PostgreSQL database: {}", config.database_url),
            )
            .await?;
        return Ok(());
    }

    let db_path = &config.database_url;

    let parent_dir = Path::new(db_path)
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Invalid database path: {}", db_path))?;

    if !parent_dir.exists() {
        std::fs::create_dir_all(parent_dir)?;
    }

    let db_url = format!("sqlite://{}?mode=rwc", db_path);

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

    logger
        .info(
            "setup",
            &format!("Database initialized at path: {}", db_path),
        )
        .await?;

    pool.close().await;

    Ok(())
}

async fn install_all_modules(logger: &LogService) -> Result<()> {
    let config = Config::from_env()?;
    let modules = Modules::load(&config.system_path)?;
    let all_modules = modules.all();

    if all_modules.is_empty() {
        logger.warn("setup", "No modules found to install").await?;
        return Ok(());
    }

    logger
        .info(
            "setup",
            &format!("Discovered {} modules to install", all_modules.len()),
        )
        .await?;

    let mut sorted_modules = all_modules.clone();
    sorted_modules.sort_by_key(|m| m.weight.unwrap_or(100));

    logger.info("setup", "Installation order:").await?;
    for module in &sorted_modules {
        logger
            .info(
                "setup",
                &format!(
                    "Module queued: {} ({}) weight={}",
                    module.name,
                    module.display_name,
                    module.weight.unwrap_or(100)
                ),
            )
            .await?;
    }

    logger.info("setup", "Installing modules...").await?;

    for module in &sorted_modules {
        logger
            .info("setup", &format!("Installing module: {}", module.name))
            .await?;
        install_module(module).await?;
    }

    logger
        .info("setup", "All modules installed successfully!")
        .await?;

    Ok(())
}

pub async fn execute(_args: SetupArgs) -> Result<()> {
    dotenvy::dotenv().ok();

    Config::init()?;

    let config = Config::from_env()?;
    let database =
        Arc::new(Database::from_config(&config.database_type, &config.database_url).await?);
    let logger = LogService::system(database.clone());

    logger
        .info("setup", "Setting up SystemPrompt OS...")
        .await?;

    initialize_database(&logger).await?;
    install_all_modules(&logger).await?;

    logger
        .info(
            "setup",
            "Setup complete! You can now run: systemprompt serve all",
        )
        .await?;

    Ok(())
}
