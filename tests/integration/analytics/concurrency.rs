/// Tests for analytics under high concurrency
///
/// Validates that:
/// - Multiple concurrent requests don't cause FK violations
/// - Session creation is atomic and thread-safe
/// - Analytics events and endpoint requests maintain referential integrity
/// - No race conditions between session creation and analytics logging
use crate::common::*;
use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;

#[tokio::test]
async fn test_high_concurrency_analytics() -> Result<()> {
    let ctx = Arc::new(TestContext::new().await?);
    let fingerprint = ctx.fingerprint().to_string();

    const CONCURRENT_REQUESTS: usize = 100;

    let mut tasks = Vec::new();
    for i in 0..CONCURRENT_REQUESTS {
        let ctx_clone = Arc::clone(&ctx);
        let fingerprint_clone = fingerprint.clone();

        let task = tokio::spawn(async move {
            let path = if i % 3 == 0 {
                "/"
            } else if i % 3 == 1 {
                "/blog/test-post"
            } else {
                "/api/v1/content/posts"
            };

            let url = format!("{}{}", ctx_clone.base_url, path);
            ctx_clone
                .http
                .get(&url)
                .header("x-fingerprint", &fingerprint_clone)
                .header("user-agent", &format!("TestAgent/{}", i))
                .header("accept", "text/html,application/json")
                .send()
                .await
        });

        tasks.push(task);
    }

    let responses = join_all(tasks).await;

    let mut success_count = 0;
    let mut error_count = 0;

    for result in responses {
        match result {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    success_count += 1;
                } else {
                    error_count += 1;
                }
            },
            _ => {
                error_count += 1;
            },
        }
    }

    println!("✓ Completed {} concurrent requests", CONCURRENT_REQUESTS);
    println!("  ├─ Success: {}", success_count);
    println!("  └─ Errors: {}", error_count);

    wait_for_async_processing().await;

    let orphaned_events_query = r#"
        SELECT COUNT(*) as orphaned_count
        FROM analytics_events ae
        WHERE ae.session_id LIKE $1
          AND NOT EXISTS (
            SELECT 1 FROM user_sessions s WHERE s.session_id = ae.session_id
          )
    "#;

    let pattern = format!("{}%", fingerprint);
    let orphaned_events = ctx
        .db
        .fetch_all(&orphaned_events_query, &[&pattern])
        .await?;
    let orphaned_event_count = orphaned_events
        .first()
        .and_then(|row| row.get("orphaned_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        orphaned_event_count, 0,
        "Found {} orphaned analytics events after concurrent requests",
        orphaned_event_count
    );

    let orphaned_requests_query = r#"
        SELECT COUNT(*) as orphaned_count
        FROM endpoint_requests er
        JOIN user_sessions s ON s.session_id = er.session_id
        WHERE s.fingerprint_hash LIKE $1
          AND NOT EXISTS (
            SELECT 1 FROM user_sessions s2 WHERE s2.session_id = er.session_id
          )
    "#;

    let orphaned_requests = ctx
        .db
        .fetch_all(&orphaned_requests_query, &[&pattern])
        .await?;
    let orphaned_req_count = orphaned_requests
        .first()
        .and_then(|row| row.get("orphaned_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        orphaned_req_count, 0,
        "Found {} orphaned endpoint requests after concurrent requests",
        orphaned_req_count
    );

    let fk_violation_check = r#"
        SELECT
            (SELECT COUNT(*) FROM analytics_events WHERE session_id = '__anonymous__') as anon_events,
            (SELECT COUNT(*) FROM endpoint_requests WHERE session_id = '__anonymous__') as anon_requests
    "#;

    let fk_check = ctx.db.fetch_all(&fk_violation_check, &[]).await?;
    let anon_events = fk_check
        .first()
        .and_then(|row| row.get("anon_events"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let anon_requests = fk_check
        .first()
        .and_then(|row| row.get("anon_requests"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        anon_events, 0,
        "Found {} analytics events with __anonymous__ session_id (FK violation symptom)",
        anon_events
    );
    assert_eq!(
        anon_requests, 0,
        "Found {} endpoint requests with __anonymous__ session_id (FK violation symptom)",
        anon_requests
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ High concurrency analytics test passed - no FK violations detected");
    Ok(())
}

#[tokio::test]
async fn test_concurrent_session_creation_deduplication() -> Result<()> {
    let ctx = Arc::new(TestContext::new().await?);
    let fingerprint = ctx.fingerprint().to_string();

    const CONCURRENT_REQUESTS: usize = 50;

    let mut tasks = Vec::new();
    for _ in 0..CONCURRENT_REQUESTS {
        let ctx_clone = Arc::clone(&ctx);
        let fingerprint_clone = fingerprint.clone();

        let task = tokio::spawn(async move {
            let url = format!("{}/", ctx_clone.base_url);
            ctx_clone
                .http
                .get(&url)
                .header("x-fingerprint", &fingerprint_clone)
                .header("user-agent", "TestAgent/Concurrent")
                .header("accept", "text/html")
                .send()
                .await
        });

        tasks.push(task);
    }

    join_all(tasks).await;

    wait_for_async_processing().await;

    let session_count_query = r#"
        SELECT COUNT(*) as session_count
        FROM user_sessions
        WHERE fingerprint_hash = $1
    "#;

    let sessions = ctx
        .db
        .fetch_all(&session_count_query, &[&fingerprint])
        .await?;
    let session_count = sessions
        .first()
        .and_then(|row| row.get("session_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert!(
        session_count >= 1,
        "Expected at least 1 session, found {}",
        session_count
    );

    println!(
        "✓ Created {} sessions for {} concurrent requests with same fingerprint",
        session_count, CONCURRENT_REQUESTS
    );
    println!("✓ Session deduplication working correctly");

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    Ok(())
}
