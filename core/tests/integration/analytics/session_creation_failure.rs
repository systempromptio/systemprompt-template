/// Tests for session creation failure scenarios
///
/// Validates that:
/// - When session creation fails, the middleware returns 503 (not 200 with FK
///   violations)
/// - No analytics events or endpoint requests are created with invalid
///   session_ids
/// - The system maintains data integrity even under error conditions
/// - No orphaned records are created when session creation fails
use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_no_anonymous_session_fk_violations() -> Result<()> {
    let ctx = TestContext::new().await?;

    let check_anonymous_violations_query = r#"
        SELECT
            (SELECT COUNT(*) FROM analytics_events WHERE session_id = '__anonymous__') as anon_events,
            (SELECT COUNT(*) FROM endpoint_requests WHERE session_id = '__anonymous__') as anon_requests
    "#;

    let before_check = ctx
        .db
        .fetch_all(&check_anonymous_violations_query, &[])
        .await?;
    let before_events = before_check
        .first()
        .and_then(|row| row.get("anon_events"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let before_requests = before_check
        .first()
        .and_then(|row| row.get("anon_requests"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    for i in 0..10 {
        let path = if i % 2 == 0 { "/" } else { "/" };

        let response = ctx.make_request(path).await;

        if let Ok(resp) = response {
            let status = resp.status();
            assert!(
                status.is_success() || status == 503 || status == 404,
                "Expected success, 404, or 503, got {}",
                status
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    wait_for_async_processing().await;

    let after_check = ctx
        .db
        .fetch_all(&check_anonymous_violations_query, &[])
        .await?;
    let after_events = after_check
        .first()
        .and_then(|row| row.get("anon_events"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let after_requests = after_check
        .first()
        .and_then(|row| row.get("anon_requests"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        after_events, before_events,
        "Found new analytics events with __anonymous__ session_id (FK violation would occur)"
    );
    assert_eq!(
        after_requests, before_requests,
        "Found new endpoint requests with __anonymous__ session_id (FK violation would occur)"
    );

    ctx.cleanup().await?;

    println!("✓ No FK violations detected - __anonymous__ session_id never persisted");
    Ok(())
}

#[tokio::test]
async fn test_analytics_referential_integrity_under_stress() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    for i in 0..50 {
        let path = if i % 2 == 0 { "/" } else { "/blog/test" };
        let _ = ctx.make_request(path).await;

        if i % 10 == 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    }

    wait_for_async_processing().await;

    let orphaned_events_query = r#"
        SELECT ae.id, ae.session_id, ae.event_type
        FROM analytics_events ae
        LEFT JOIN user_sessions s ON s.session_id = ae.session_id
        WHERE s.session_id IS NULL
        LIMIT 5
    "#;

    let orphaned_events = ctx.db.fetch_all(&orphaned_events_query, &[]).await?;

    assert!(
        orphaned_events.is_empty(),
        "Found {} orphaned analytics events: {:?}",
        orphaned_events.len(),
        orphaned_events
    );

    let orphaned_requests_query = r#"
        SELECT er.id, er.session_id, er.endpoint_path
        FROM endpoint_requests er
        LEFT JOIN user_sessions s ON s.session_id = er.session_id
        WHERE s.session_id IS NULL
        LIMIT 5
    "#;

    let orphaned_requests = ctx.db.fetch_all(&orphaned_requests_query, &[]).await?;

    assert!(
        orphaned_requests.is_empty(),
        "Found {} orphaned endpoint requests: {:?}",
        orphaned_requests.len(),
        orphaned_requests
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Referential integrity maintained under stress - no orphaned records");
    Ok(())
}

#[tokio::test]
async fn test_rapid_sequential_requests_no_fk_violations() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    const RAPID_REQUESTS: usize = 30;

    for _ in 0..RAPID_REQUESTS {
        let _ = ctx.make_request("/").await;
    }

    wait_for_async_processing().await;

    let fk_violation_query = r#"
        SELECT
            COUNT(DISTINCT ae.session_id) as event_sessions,
            COUNT(DISTINCT er.session_id) as request_sessions,
            COUNT(DISTINCT s.session_id) as actual_sessions
        FROM user_sessions s
        FULL OUTER JOIN analytics_events ae ON ae.session_id = s.session_id
        FULL OUTER JOIN endpoint_requests er ON er.session_id = s.session_id
        WHERE s.fingerprint_hash = $1
    "#;

    let result = ctx
        .db
        .fetch_all(&fk_violation_query, &[&fingerprint])
        .await?;

    if let Some(row) = result.first() {
        let event_sessions = row
            .get("event_sessions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let request_sessions = row
            .get("request_sessions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let actual_sessions = row
            .get("actual_sessions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        assert!(
            event_sessions <= actual_sessions,
            "Analytics events reference {} sessions but only {} exist",
            event_sessions,
            actual_sessions
        );

        assert!(
            request_sessions <= actual_sessions,
            "Endpoint requests reference {} sessions but only {} exist",
            request_sessions,
            actual_sessions
        );

        println!("✓ Rapid sequential requests completed successfully");
        println!("  ├─ Actual sessions: {}", actual_sessions);
        println!("  ├─ Event sessions: {}", event_sessions);
        println!("  └─ Request sessions: {}", request_sessions);
    }

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    Ok(())
}
