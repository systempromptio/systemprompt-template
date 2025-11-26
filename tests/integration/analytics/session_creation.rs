/// Tests for analytics session creation
///
/// Tests:
/// - Anonymous session creation on homepage
/// - Session fingerprint deduplication
/// - Session with authenticated user
/// - Session with UTM parameters
use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_anonymous_session_created_on_homepage() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success(), "HTTP request failed");

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(
        !rows.is_empty(),
        "No session created for fingerprint: {}",
        fingerprint
    );

    let session = crate::common::context::get_session_from_row(&rows[0])?;
    assert_eq!(session.fingerprint_hash, Some(fingerprint.clone()));
    assert_eq!(session.user_type, "anon");
    assert_eq!(session.request_count, 1);
    assert!(session.started_at <= chrono::Utc::now());

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Anonymous session created and verified");
    Ok(())
}

#[tokio::test]
async fn test_session_fingerprint_deduplication() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    for _ in 0..3 {
        let response = ctx.make_request("/").await?;
        assert!(response.status().is_success());
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert_eq!(
        rows.len(),
        1,
        "Multiple sessions created for same fingerprint"
    );

    let session = crate::common::context::get_session_from_row(&rows[0])?;
    assert_eq!(
        session.request_count, 3,
        "Request count didn't increment to 3"
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Fingerprint deduplication verified");
    Ok(())
}

#[tokio::test]
async fn test_session_with_utm_parameters() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let path = "/?utm_source=google&utm_medium=organic&utm_campaign=product-launch";
    let response = ctx.make_request(path).await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "Session not created");
    let session = crate::common::context::get_session_from_row(&rows[0])?;

    assert_eq!(session.utm_source, Some("google".to_string()));
    assert_eq!(session.utm_medium, Some("organic".to_string()));
    assert_eq!(session.utm_campaign, Some("product-launch".to_string()));

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ UTM parameters tracked correctly");
    Ok(())
}

#[tokio::test]
async fn test_landing_page_captured_on_first_request() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let landing_path = "/";
    let response = ctx.make_request(landing_path).await?;
    assert!(
        response.status().is_success(),
        "HTTP request failed for landing page test"
    );

    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(
        !rows.is_empty(),
        "Session not created for landing page test"
    );

    let session = crate::common::context::get_session_from_row(&rows[0])?;

    assert!(
        session.landing_page.is_some(),
        "BUG: landing_page should not be NULL for HTML pages. Got: {:?}",
        session.landing_page
    );

    let actual_landing_page = session.landing_page.as_ref().unwrap();
    assert_eq!(
        actual_landing_page, landing_path,
        "BUG: landing_page should be '{}' but got '{}'",
        landing_path, actual_landing_page
    );

    assert!(
        session.entry_url.is_some(),
        "BUG: entry_url should not be NULL. Got: {:?}",
        session.entry_url
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Landing page captured correctly");
    Ok(())
}

#[tokio::test]
async fn test_landing_page_preserved_across_multiple_requests() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let landing_path = "/";
    ctx.make_request(landing_path).await?;
    wait_for_async_processing().await;

    ctx.make_request("/api/v1/content/blog").await?;
    wait_for_async_processing().await;

    ctx.make_request("/api/v1/content/web").await?;
    wait_for_async_processing().await;

    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "Session not created");
    let session = crate::common::context::get_session_from_row(&rows[0])?;

    assert_eq!(
        session.request_count, 3,
        "Expected 3 requests, got {}",
        session.request_count
    );

    assert!(
        session.landing_page.is_some(),
        "BUG: landing_page should not be NULL after multiple requests (including API)"
    );

    let actual_landing_page = session.landing_page.as_ref().unwrap();
    assert_eq!(
        actual_landing_page, landing_path,
        "BUG: landing_page changed from initial value. Expected '{}', got '{}'",
        landing_path, actual_landing_page
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Landing page preserved across multiple requests");
    Ok(())
}
