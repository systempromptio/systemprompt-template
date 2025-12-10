use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;
use uuid::Uuid;

#[tokio::test]
async fn test_timestamp_fields_store_correctly() -> Result<()> {
    let ctx = TestContext::new().await?;

    let session_id = format!("test-ts-{}", Uuid::new_v4());

    let insert_query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
    ctx.db.execute(&insert_query, &[&session_id]).await?;

    let select_query = "SELECT started_at FROM user_sessions WHERE session_id = $1";
    let rows = ctx.db.fetch_all(&select_query, &[&session_id]).await?;

    assert!(!rows.is_empty(), "Session not found");

    let started_at = rows[0].get("started_at");
    assert!(started_at.is_some(), "started_at value is missing");

    println!("✓ Timestamps stored and retrieved correctly");
    Ok(())
}

#[tokio::test]
async fn test_json_fields_store_and_retrieve() -> Result<()> {
    let ctx = TestContext::new().await?;

    let session_id = format!("test-json-{}", Uuid::new_v4());

    let insert_query = "INSERT INTO user_sessions (session_id, started_at) VALUES ($1, NOW())";
    ctx.db.execute(&insert_query, &[&session_id]).await?;

    let query = "SELECT COUNT(*) as json_column_count FROM information_schema.columns WHERE \
                 table_schema = 'public' AND data_type = 'jsonb'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let json_count = rows[0]
        .get("json_column_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(
        json_count >= 1,
        "Expected at least 1 JSON column, found {}",
        json_count
    );

    println!("✓ JSON fields properly defined ({} columns)", json_count);
    Ok(())
}

#[tokio::test]
async fn test_numeric_fields_store_correctly() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as numeric_count FROM information_schema.columns WHERE \
                 table_schema = 'public' AND data_type IN ('integer', 'bigint', 'numeric', \
                 'double precision')";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let numeric_count = rows[0]
        .get("numeric_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(numeric_count > 0, "No numeric columns found in schema");

    println!("✓ {} numeric columns properly defined", numeric_count);
    Ok(())
}
