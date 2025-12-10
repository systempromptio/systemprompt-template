use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;
use uuid::Uuid;

#[tokio::test]
async fn test_foreign_keys_enforced() -> Result<()> {
    let ctx = TestContext::new().await?;

    let fake_session_id = format!("non-existent-{}", Uuid::new_v4());

    let insert_query = "INSERT INTO analytics_events
        (session_id, event_type, event_category, severity)
        VALUES ($1, $2, $3, $4)";

    let result = ctx
        .db
        .execute(
            &insert_query,
            &[&fake_session_id, &"page_view", &"user_action", &"info"],
        )
        .await;

    assert!(result.is_err(), "Foreign key constraint was not enforced");
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("foreign key") || error_msg.contains("violates"),
        "Wrong error type for foreign key violation: {}",
        error_msg
    );

    println!("✓ Foreign key constraints enforced");
    Ok(())
}

#[tokio::test]
async fn test_foreign_keys_exist() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as fk_count FROM information_schema.table_constraints WHERE \
                 table_schema = 'public' AND constraint_type = 'FOREIGN KEY'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let fk_count = rows[0]
        .get("fk_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(fk_count > 0, "No foreign key constraints found");

    println!("✓ {} foreign key constraints found", fk_count);
    Ok(())
}
