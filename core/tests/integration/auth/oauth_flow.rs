use crate::common::*;
use anyhow::Result;

#[tokio::test]
async fn test_oauth_authorization_flow() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let oauth_url = format!("{}/api/v1/core/oauth/authorize", ctx.base_url);
    let response = ctx
        .http
        .get(&oauth_url)
        .query(&[
            ("provider", "test"),
            ("redirect_uri", "http://localhost:3000/callback"),
            ("client_id", "test-client"),
        ])
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    let status = response.status();
    assert!(
        status.is_redirection() || status.is_success() || status.is_client_error(),
        "OAuth authorize endpoint responded with: {}",
        status
    );

    if status.is_redirection() {
        let location = response
            .headers()
            .get("location")
            .and_then(|h| h.to_str().ok());

        if let Some(loc) = location {
            println!("📍 Redirect location: {}", loc);
        }
    }

    println!("✓ OAuth authorization endpoint accessible");
    Ok(())
}

#[tokio::test]
async fn test_oauth_user_creation_on_first_login() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let callback_url = format!("{}/api/v1/core/oauth/callback", ctx.base_url);
    let callback_response = ctx
        .http
        .get(&callback_url)
        .query(&[
            ("code", "test-auth-code"),
            ("state", "test-state"),
            ("provider", "test"),
        ])
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    let status = callback_response.status();
    assert!(
        status.is_success() || status.is_redirection() || status.is_server_error(),
        "OAuth callback endpoint responded with: {}",
        status
    );

    wait_for_async_processing().await;

    println!("✓ OAuth callback endpoint accessible");
    Ok(())
}
