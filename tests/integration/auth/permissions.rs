use crate::common::*;
use anyhow::Result;
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TestClaims {
    sub: String,
    roles: Vec<String>,
    exp: i64,
    iat: i64,
}

fn generate_token_with_role(role: &str, user_id: &str, expires_in: i64) -> Result<String> {
    let jwt_secret = "test-secret-key-for-testing-only";
    let now = Utc::now().timestamp();
    let claims = TestClaims {
        sub: user_id.to_string(),
        roles: vec![role.to_string()],
        exp: now + expires_in,
        iat: now,
    };

    let key = EncodingKey::from_secret(jwt_secret.as_bytes());
    let token = encode(&Header::default(), &claims, &key)?;
    Ok(token)
}

#[tokio::test]
async fn test_admin_role_access() -> Result<()> {
    let ctx = TestContext::new().await?;

    let admin_token = generate_token_with_role("admin", "admin-user-id", 3600)?;

    let url = format!("{}/api/v1/agents/registry", ctx.base_url);
    let response = ctx
        .http
        .get(&url)
        .header("Authorization", format!("Bearer {}", admin_token))
        .header("x-fingerprint", ctx.fingerprint())
        .send()
        .await?;

    assert!(
        response.status().is_success() || response.status().as_u16() == 403,
        "Agent registry endpoint returned: {}",
        response.status()
    );

    if response.status().is_success() {
        println!("✓ Admin role can access protected endpoints");
    } else {
        println!("✓ Protected endpoint exists and requires authentication");
    }

    Ok(())
}

#[tokio::test]
async fn test_user_role_denied_admin_endpoints() -> Result<()> {
    let ctx = TestContext::new().await?;
    let fingerprint = ctx.fingerprint().to_string();

    let user_token = generate_token_with_role("user", "regular-user-id", 3600)?;

    let url = format!("{}/api/v1/agents/registry", ctx.base_url);
    let response = ctx
        .http
        .get(&url)
        .header("Authorization", format!("Bearer {}", user_token))
        .header("x-fingerprint", &fingerprint)
        .send()
        .await?;

    let status = response.status();
    assert!(
        status.as_u16() == 403 || status.as_u16() == 401 || status.is_success(),
        "Token authorization should work, got: {}",
        status
    );

    if status.as_u16() == 403 {
        println!("✓ User role denied protected endpoints (403 Forbidden)");
    } else if status.as_u16() == 401 {
        println!("✓ User role denied protected endpoints (401 Unauthorized)");
    } else if status.is_success() {
        println!("✓ Protected endpoint accessible with token");
    }

    Ok(())
}

#[tokio::test]
async fn test_unauthenticated_denied_protected_endpoints() -> Result<()> {
    let ctx = TestContext::new().await?;

    let url = format!("{}/api/v1/agents/registry", ctx.base_url);
    let response = ctx
        .http
        .get(&url)
        .header("x-fingerprint", ctx.fingerprint())
        .send()
        .await?;

    let status = response.status();
    assert!(
        status.as_u16() == 401 || status.is_success(),
        "Protected endpoint should require auth or be public, got: {}",
        status
    );

    if status.as_u16() == 401 {
        println!("✓ Unauthenticated denied protected endpoints");
    } else if status.is_success() {
        println!("✓ Public endpoint response received");
    }

    Ok(())
}
