use crate::models::context::AppContext;
use anyhow::{bail, Context, Result};
use std::path::Path;

pub async fn validate_database(ctx: &AppContext) -> Result<()> {
    validate_database_path(&ctx.config().database_url)?;
    validate_database_connection(ctx).await?;
    Ok(())
}

fn validate_database_path(db_path: &str) -> Result<()> {
    if db_path.is_empty() {
        bail!("DATABASE_URL is empty");
    }

    // Skip file validation for PostgreSQL connection strings
    if db_path.starts_with("postgresql://") || db_path.starts_with("postgres://") {
        return Ok(());
    }

    // SQLite: validate file exists
    let path = Path::new(db_path);

    if !path.exists() {
        bail!("Database not found at '{db_path}'. Run setup first");
    }

    if !path.is_file() {
        bail!("Database path '{db_path}' exists but is not a file");
    }

    Ok(())
}

async fn validate_database_connection(ctx: &AppContext) -> Result<()> {
    ctx.db_pool()
        .test_connection()
        .await
        .context("Failed to establish database connection")?;
    Ok(())
}
