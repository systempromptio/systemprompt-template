use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseProvider;

#[tokio::test]
async fn test_postgres_connection_pool_established() -> Result<()> {
    let ctx = TestContext::new().await?;

    let query = "SELECT 1 as test_value";
    let rows = ctx.db.fetch_all(&query, &[]).await?;

    assert!(!rows.is_empty(), "No rows returned from test query");
    assert!(
        rows[0].get("test_value").is_some(),
        "Missing test_value column"
    );

    println!("✓ PostgreSQL connection pool established");
    Ok(())
}

#[tokio::test]
async fn test_database_schema_initialized() -> Result<()> {
    let ctx = TestContext::new().await?;

    let required_tables = vec![
        "user_sessions",
        "analytics_events",
        "agent_tasks",
        "task_messages",
        "ai_requests",
        "endpoint_requests",
        "markdown_content",
        "services",
    ];

    for table in required_tables {
        let query = "SELECT 1 FROM information_schema.tables WHERE table_schema = 'public' AND \
                     table_name = $1";
        let rows = ctx.db.fetch_all(&query, &[&table]).await?;

        assert!(!rows.is_empty(), "Table '{}' does not exist", table);
    }

    println!("✓ All required tables exist");
    Ok(())
}
