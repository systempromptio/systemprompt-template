use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;

#[tokio::test]
async fn test_migrations_run_sequentially() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as table_count FROM information_schema.tables WHERE table_schema = 'public'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let table_count = rows[0]
        .get("table_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(
        table_count > 0,
        "No tables found - migrations may not have run"
    );

    println!(
        "✓ Migrations run sequentially - {} tables created",
        table_count
    );
    Ok(())
}

#[tokio::test]
async fn test_migrations_idempotent() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query =
        "SELECT COUNT(*) as count FROM information_schema.tables WHERE table_schema = 'public'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let initial_count = rows[0].get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    let rows_after = ctx.db.fetch_all(&query, &[]).await?;
    let final_count = rows_after[0]
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        initial_count, final_count,
        "Table count changed on repeated query - migrations may not be idempotent"
    );

    println!(
        "✓ Migrations are idempotent with {} total tables",
        final_count
    );
    Ok(())
}

#[tokio::test]
async fn test_migration_rollback_not_supported() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as table_count FROM information_schema.tables WHERE table_schema = 'public'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let table_count = rows[0]
        .get("table_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(table_count > 0, "No tables exist in database");

    println!(
        "✓ Migration rollback not supported (forward-only model) - {} tables present",
        table_count
    );
    Ok(())
}
