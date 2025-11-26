/// Content Config and Landing Page Integration Tests
///
/// These tests verify that:
/// 1. CONTENT_CONFIG_PATH environment variable is set
/// 2. ContentConfig is properly loaded by AppContext
/// 3. Landing pages are correctly identified for different URL patterns
/// 4. Landing page tracking fails gracefully if content_config is None
///
/// Root Cause: Production startup script (startup.sh) was missing CONTENT_CONFIG_PATH
/// in the .env.docker file, causing content_config to be None and all landing page
/// tracking to fail silently (resulting in "(not set)" in analytics dashboard).

use crate::common::*;
use anyhow::Result;
use systemprompt_core_database::DatabaseQueryEnum;

#[tokio::test]
async fn test_content_config_path_env_var_is_set() -> Result<()> {
    // CRITICAL: This test ensures CONTENT_CONFIG_PATH is set in production
    let content_config_path = std::env::var("CONTENT_CONFIG_PATH");

    assert!(
        content_config_path.is_ok(),
        "CRITICAL BUG: CONTENT_CONFIG_PATH environment variable is not set. \
        This causes all landing page tracking to fail silently. \
        Add CONTENT_CONFIG_PATH to production .env.docker file."
    );

    let path = content_config_path.unwrap();
    assert!(
        !path.is_empty(),
        "CRITICAL BUG: CONTENT_CONFIG_PATH is set but empty"
    );

    println!("✓ CONTENT_CONFIG_PATH is set: {}", path);
    Ok(())
}

#[tokio::test]
async fn test_app_context_loads_content_config() -> Result<()> {
    use systemprompt_core_system::AppContext;

    // This will panic if CONTENT_CONFIG_PATH is not set
    let app_context = AppContext::new().await?;

    // Check if content_config is loaded
    let content_config = app_context.content_config();

    assert!(
        content_config.is_some(),
        "CRITICAL BUG: AppContext.content_config() is None. \
        This means the content config file couldn't be loaded. \
        Verify CONTENT_CONFIG_PATH points to a valid config file. \
        Without this, all landing page tracking fails."
    );

    println!("✓ AppContext successfully loaded content_config");
    Ok(())
}

#[tokio::test]
async fn test_landing_page_set_for_homepage() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Request homepage
    let response = ctx.make_request("/").await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    // Verify landing page is set
    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "No session created");

    let session = &rows[0];
    let landing_page = session
        .get("landing_page")
        .and_then(|v| v.as_str());

    assert!(
        landing_page.is_some(),
        "CRITICAL BUG: landing_page is NULL for homepage request. \
        Likely cause: content_config is None or CONTENT_CONFIG_PATH not set."
    );

    assert_eq!(
        landing_page.unwrap(),
        "/",
        "landing_page should be '/' for homepage"
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Homepage landing page correctly set");
    Ok(())
}

#[tokio::test]
async fn test_landing_page_set_for_blog_post() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Request a blog post URL
    let blog_url = "/blog/mcp-reckoning";
    let response = ctx.make_request(blog_url).await?;

    // Even if the blog post doesn't exist, the session should be created
    // and landing_page should be set if it's recognized as an HTML page

    wait_for_async_processing().await;

    // Verify landing page is set
    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "No session created");

    let session = &rows[0];
    let landing_page = session
        .get("landing_page")
        .and_then(|v| v.as_str());

    assert!(
        landing_page.is_some(),
        "CRITICAL BUG: landing_page is NULL for blog post request '{}'. \
        Likely causes:\n\
        1. content_config is None (CONTENT_CONFIG_PATH not set)\n\
        2. Blog URL pattern not configured in config.yml\n\
        3. is_html_page() returning false incorrectly",
        blog_url
    );

    assert_eq!(
        landing_page.unwrap(),
        blog_url,
        "landing_page should match the requested blog URL"
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Blog post landing page correctly set: {}", blog_url);
    Ok(())
}

#[tokio::test]
async fn test_landing_page_not_set_for_api_requests() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Request API endpoint
    let api_url = "/api/v1/health";
    let response = ctx.make_request(api_url).await?;
    assert!(response.status().is_success());

    wait_for_async_processing().await;

    // Verify landing page is NOT set for API requests
    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "No session created");

    let session = &rows[0];
    let landing_page = session
        .get("landing_page")
        .and_then(|v| v.as_str());

    // API requests should NOT set landing_page
    assert!(
        landing_page.is_none() || landing_page.unwrap().is_empty(),
        "BUG: landing_page should not be set for API requests, but got: {:?}",
        landing_page
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ API request correctly did not set landing_page");
    Ok(())
}

#[tokio::test]
async fn test_landing_page_set_for_legal_pages() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    // Request legal page
    let legal_url = "/privacy-policy";
    let response = ctx.make_request(legal_url).await?;

    wait_for_async_processing().await;

    // Verify landing page is set
    let query = DatabaseQueryEnum::FindSessionByFingerprintAny.get(ctx.db.as_ref());
    let rows = ctx.db.fetch_all(&query, &[&fingerprint]).await?;

    assert!(!rows.is_empty(), "No session created");

    let session = &rows[0];
    let landing_page = session
        .get("landing_page")
        .and_then(|v| v.as_str());

    assert!(
        landing_page.is_some(),
        "CRITICAL BUG: landing_page is NULL for legal page '{}'. \
        Likely causes:\n\
        1. content_config is None (CONTENT_CONFIG_PATH not set)\n\
        2. Legal page URL pattern not configured in config.yml",
        legal_url
    );

    assert_eq!(
        landing_page.unwrap(),
        legal_url,
        "landing_page should match the requested legal page URL"
    );

    let mut cleanup = TestCleanup::new(ctx.db.clone());
    cleanup.track_fingerprint(fingerprint);
    cleanup.cleanup_all().await?;

    println!("✓ Legal page landing page correctly set: {}", legal_url);
    Ok(())
}
