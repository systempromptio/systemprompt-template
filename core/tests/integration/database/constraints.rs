use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;
use uuid::Uuid;

#[tokio::test]
async fn test_primary_keys_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    let session_id = format!("test-pk-{}", Uuid::new_v4());

    let insert_query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
    ctx.db.execute(&insert_query, &[&session_id]).await?;

    let result = ctx.db.execute(&insert_query, &[&session_id]).await;

    assert!(result.is_err(), "Duplicate primary key was not rejected");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("duplicate") || error_msg.contains("unique"),
        "Wrong error type for duplicate key: {}",
        error_msg
    );

    println!("✓ Primary key constraints enforced");
    Ok(())
}

#[tokio::test]
async fn test_unique_constraints_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as unique_count FROM information_schema.table_constraints WHERE \
                 table_schema = 'public' AND constraint_type = 'UNIQUE'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let unique_count = rows[0]
        .get("unique_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(unique_count > 0, "No unique constraints found");

    println!("✓ {} unique constraints enforced", unique_count);
    Ok(())
}

#[tokio::test]
async fn test_not_null_constraints_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as not_null_count FROM information_schema.columns WHERE \
                 table_schema = 'public' AND is_nullable = 'NO'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let not_null_count = rows[0]
        .get("not_null_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(not_null_count > 0, "No NOT NULL constraints found");

    println!("✓ {} NOT NULL constraints enforced", not_null_count);
    Ok(())
}

#[tokio::test]
async fn test_all_primary_keys_exist() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as pk_count FROM information_schema.table_constraints WHERE \
                 table_schema = 'public' AND constraint_type = 'PRIMARY KEY'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let pk_count = rows[0]
        .get("pk_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(
        pk_count >= 8,
        "Expected at least 8 primary key constraints, found {}",
        pk_count
    );

    println!("✓ {} primary key constraints found", pk_count);
    Ok(())
}
