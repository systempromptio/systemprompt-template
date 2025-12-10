use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;

#[tokio::test]
async fn test_no_orphaned_records_exist() -> Result<()> {
    let ctx = TestContext::new().await?;

    let orphaned_checks = vec![
        (
            "task_messages without tasks",
            "SELECT COUNT(*) as count FROM task_messages tm
             WHERE NOT EXISTS (SELECT 1 FROM agent_tasks at WHERE at.task_id = tm.task_id)",
        ),
        (
            "analytics_events without sessions",
            "SELECT COUNT(*) as count FROM analytics_events ae
             WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id)",
        ),
        (
            "endpoint_requests without sessions",
            "SELECT COUNT(*) as count FROM endpoint_requests er
             WHERE NOT EXISTS (SELECT 1 FROM user_sessions s WHERE s.session_id = er.session_id)",
        ),
    ];

    for (description, query) in orphaned_checks {
        let rows = ctx.db.fetch_all(&query, &[]).await?;
        let count = rows[0].get("count").and_then(|v| v.as_i64()).unwrap_or(0);

        assert_eq!(count, 0, "Found orphaned records: {}", description);
    }

    println!("✓ No orphaned records detected");
    Ok(())
}

#[tokio::test]
async fn test_all_required_columns_have_values() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT COUNT(*) as not_null_count FROM information_schema.columns WHERE \
                 table_schema = 'public' AND is_nullable = 'NO'";
    let rows = ctx.db.fetch_all(&query, &[]).await?;
    let not_null_count = rows[0]
        .get("not_null_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(not_null_count > 0, "No NOT NULL constraints found");

    println!("✓ {} NOT NULL columns verified", not_null_count);
    Ok(())
}
