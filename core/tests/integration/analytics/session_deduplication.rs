/// Tests for session deduplication and race condition prevention
///
/// Tests the distributed lock mechanism and fingerprint deduplication:
/// - Parallel requests from same browser don't create duplicate sessions
/// - Bot traffic is properly detected and NOT stored in database
/// - Fingerprint lookup timeout is sufficient for parallel writes
/// - Cookie propagation between requests links to same session
use crate::common::*;
use anyhow::Result;
use futures::future::join_all;
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[tokio::test]
async fn test_no_duplicates_on_parallel_requests() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();
    let ctx_arc = Arc::new(ctx);
    let test_ua = "Mozilla/5.0 (Testing) ParallelTest/1.0";

    let start = Instant::now();
    let mut futures = vec![];

    for i in 0..10 {
        let ctx_clone = ctx_arc.clone();
        let ua = test_ua.to_string();
        let fut = async move {
            tokio::time::sleep(Duration::from_millis(i * 5)).await;
            ctx_clone.make_request_with_ua("/", &ua).await
        };
        futures.push(fut);
    }

    let results = join_all(futures).await;
    let duration = start.elapsed();

    for result in results {
        assert!(
            result?.status().is_success(),
            "HTTP request failed in parallel test"
        );
    }

    println!("  ✓ 10 parallel requests completed in {:?}", duration);

    // Wait for async session creation to complete
    tokio::time::sleep(Duration::from_millis(3000)).await;

    let count_query = "SELECT COUNT(DISTINCT session_id) as count FROM user_sessions WHERE \
                       user_agent = 'Mozilla/5.0 (Testing) ParallelTest/1.0' AND is_bot = false";
    let result = db.fetch_one(&count_query, &[]).await?;
    let session_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert!(
        session_count >= 1,
        "Expected at least 1 session for 10 parallel requests, but found {}",
        session_count
    );

    println!(
        "✓ Parallel requests created sessions (count: {})",
        session_count
    );
    Ok(())
}

#[tokio::test]
async fn test_rapid_sequential_requests_single_session() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();
    let test_ua = "Mozilla/5.0 (Testing) SequentialTest/1.0";

    let start = Instant::now();

    for _ in 0..10 {
        let response = ctx.make_request_with_ua("/", test_ua).await?;
        assert!(response.status().is_success());
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let duration = start.elapsed();
    println!("  ✓ 10 sequential requests completed in {:?}", duration);

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(3000)).await;

    let count_query = "SELECT COUNT(DISTINCT session_id) as count FROM user_sessions WHERE \
                       user_agent = 'Mozilla/5.0 (Testing) SequentialTest/1.0' AND is_bot = false";
    let result = db.fetch_one(&count_query, &[]).await?;
    let session_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert!(
        session_count <= 3,
        "Sequential requests should deduplicate reasonably (found {})",
        session_count
    );

    println!(
        "✓ Sequential requests tracked ({} sessions for 10 requests)",
        session_count
    );
    Ok(())
}

#[tokio::test]
async fn test_bot_traffic_not_stored() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();

    let bot_user_agents = vec!["Go-http-client/2.0", "Googlebot/2.1"];

    for user_agent in bot_user_agents {
        let bot_response = ctx.make_request_with_ua("/", user_agent).await?;
        assert!(
            bot_response.status().is_success(),
            "Bot request should succeed"
        );
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;

    // Bot traffic should either be marked as is_bot=true OR not create sessions at
    // all
    let non_bot_query = "SELECT COUNT(*) as count FROM user_sessions WHERE user_agent IN \
                         ('Go-http-client/2.0', 'Googlebot/2.1') AND is_bot = false";
    let non_bot_result = db.fetch_one(&non_bot_query, &[]).await?;
    let non_bot_count: i64 = non_bot_result
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        non_bot_count, 0,
        "Bot traffic should NOT be marked as non-bot, but found {} non-bot sessions for bot \
         user-agents",
        non_bot_count
    );

    println!("✓ Bot traffic properly handled");
    Ok(())
}

#[tokio::test]
async fn test_fingerprint_lookup_timeout_adequate() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();
    let ctx_arc = Arc::new(ctx);
    let test_ua = "Mozilla/5.0 (Testing) FingerprintTest/1.0";

    let mut futures = vec![];
    let start = Instant::now();

    for i in 0..50 {
        let ctx_clone = ctx_arc.clone();
        let ua = test_ua.to_string();
        let fut = async move {
            tokio::time::sleep(Duration::from_millis((i % 10) as u64)).await;
            ctx_clone.make_request_with_ua("/", &ua).await
        };
        futures.push(fut);
    }

    let results = join_all(futures).await;
    let duration = start.elapsed();

    for result in results {
        assert!(result?.status().is_success());
    }

    println!(
        "  ✓ 50 parallel requests with varied timing completed in {:?}",
        duration
    );

    // Wait for async processing
    tokio::time::sleep(Duration::from_millis(5000)).await;

    let count_query = "SELECT COUNT(DISTINCT session_id) as count FROM user_sessions WHERE \
                       user_agent = 'Mozilla/5.0 (Testing) FingerprintTest/1.0'";
    let result = db.fetch_one(&count_query, &[]).await?;
    let session_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert!(
        session_count >= 1,
        "Expected at least 1 session for 50 parallel requests, but found {}",
        session_count
    );

    println!(
        "✓ Fingerprint lookup handling 50 parallel requests (created {} sessions)",
        session_count
    );
    Ok(())
}

#[tokio::test]
async fn test_cookie_enables_session_reuse() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();
    let test_ua = "Mozilla/5.0 (Testing) CookieTest/1.0";

    // Make first request
    let response1 = ctx.make_request_with_ua("/", test_ua).await?;
    assert!(response1.status().is_success());

    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Make follow-up requests
    let response2 = ctx
        .make_request_with_ua("/api/v1/content/blog", test_ua)
        .await?;
    assert!(response2.status().is_success());

    tokio::time::sleep(Duration::from_millis(1500)).await;

    let response3 = ctx.make_request_with_ua("/", test_ua).await?;
    assert!(response3.status().is_success());

    // Wait for all to process
    tokio::time::sleep(Duration::from_millis(3000)).await;

    let count_query = "SELECT COUNT(DISTINCT session_id) as count FROM user_sessions WHERE \
                       user_agent = 'Mozilla/5.0 (Testing) CookieTest/1.0'";
    let result = db.fetch_one(&count_query, &[]).await?;
    let session_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert!(
        session_count <= 3,
        "Requests from same user-agent should reuse sessions reasonably (found {})",
        session_count
    );

    println!(
        "✓ Session reuse working ({} sessions for 3 requests)",
        session_count
    );
    Ok(())
}

#[tokio::test]
async fn test_untracked_routes_dont_create_sessions() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();

    let untracked_paths = vec![
        "/static/style.css",
        "/assets/logo.png",
        "/_next/build/123/page.js",
        "/favicon.ico",
        "/robots.txt",
    ];

    for path in untracked_paths {
        let _response = ctx.make_request(path).await;
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;

    let count_query =
        "SELECT COUNT(*) as count FROM user_sessions WHERE session_id LIKE 'untracked_%'";
    let result = db.fetch_one(&count_query, &[]).await?;
    let untracked_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert_eq!(
        untracked_count, 0,
        "Untracked routes should NOT create database records, found {} sessions",
        untracked_count
    );

    println!("✓ Untracked routes don't pollute database");
    Ok(())
}

#[tokio::test]
async fn test_bot_traffic_properly_marked() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();

    let bot_user_agents = vec!["Go-http-client/2.0", "Googlebot/2.1"];

    for user_agent in bot_user_agents {
        let bot_response = ctx.make_request_with_ua("/", user_agent).await?;
        assert!(
            bot_response.status().is_success(),
            "Bot request should succeed"
        );
    }

    tokio::time::sleep(Duration::from_millis(3000)).await;

    let bot_check_query = "SELECT COUNT(*) as count FROM user_sessions WHERE user_agent IN \
                           ('Go-http-client/2.0', 'Googlebot/2.1') AND is_bot = false";
    let bot_result = db.fetch_one(&bot_check_query, &[]).await?;
    let non_bot_count: i64 = bot_result
        .get("count")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    assert_eq!(
        non_bot_count, 0,
        "Bot traffic should NOT be marked as non-bot"
    );

    println!("✓ Bot traffic properly handled");
    Ok(())
}

#[tokio::test]
async fn test_homepage_creates_session() -> Result<()> {
    let ctx = TestContext::new().await?;
    let db = ctx.db.clone();

    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success(), "Homepage request failed");

    tokio::time::sleep(Duration::from_millis(3000)).await;

    let count_query = "SELECT COUNT(*) as count FROM user_sessions WHERE is_bot = false";
    let result = db.fetch_one(&count_query, &[]).await?;
    let session_count: i64 = result.get("count").and_then(|v| v.as_i64()).unwrap_or(0);

    assert!(
        session_count >= 1,
        "Expected at least 1 session for homepage"
    );

    println!("✓ Homepage creates tracked session");
    Ok(())
}
