/// Tests for analytics data integrity
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_analytics_referential_integrity() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    let orphaned_events_query = r#"
        SELECT COUNT(*) as orphaned_count
        FROM analytics_events ae
        WHERE NOT EXISTS (
            SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id
        )
    "#;

    let orphaned_events = ctx.db.fetch_all(&orphaned_events_query, &[]).await?;
    let orphaned_count = orphaned_events
        .first()
        .and_then(|row| row.get("orphaned_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        orphaned_count, 0,
        "Found {} orphaned analytics events",
        orphaned_count
    );

    let orphaned_requests_query = r#"
        SELECT COUNT(*) as orphaned_count
        FROM endpoint_requests er
        WHERE NOT EXISTS (
            SELECT 1 FROM user_sessions s WHERE s.session_id = er.session_id
        )
    "#;

    let orphaned_requests = ctx.db.fetch_all(&orphaned_requests_query, &[]).await?;
    let orphaned_req_count = orphaned_requests
        .first()
        .and_then(|row| row.get("orphaned_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        orphaned_req_count, 0,
        "Found {} orphaned endpoint requests",
        orphaned_req_count
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Analytics referential integrity verified");
    Ok(())
}
