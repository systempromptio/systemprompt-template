use super::setup_test_pool;
use chrono::Utc;
use systemprompt_core_oauth::repository::OAuthRepository;
use uuid::Uuid;

#[tokio::test]
#[ignore]
async fn test_authorization_code_lifecycle() {
    let pool = setup_test_pool().await;
    let repo = OAuthRepository::new(pool);

    let code = "test_auth_code_123";
    let client_id = "test_client_code";
    let user_id = Uuid::new_v4().to_string();
    let redirect_uri = "http://localhost:3000/callback";
    let scopes = "openid profile";

    repo.store_authorization_code(code, client_id, &user_id, redirect_uri, scopes, None, None)
        .await
        .expect("Failed to store authorization code");

    let stored_client_id = repo
        .get_client_id_from_auth_code(code)
        .await
        .expect("Failed to get client_id from code")
        .expect("Code not found");

    assert_eq!(stored_client_id, client_id);

    let (returned_user_id, returned_scope) = repo
        .validate_authorization_code(code, client_id, Some(redirect_uri), None)
        .await
        .expect("Failed to validate authorization code");

    assert_eq!(returned_user_id, user_id);
    assert_eq!(returned_scope, scopes);

    let validation_again = repo
        .validate_authorization_code(code, client_id, Some(redirect_uri), None)
        .await;

    assert!(
        validation_again.is_err(),
        "Should not be able to use code twice"
    );
}

#[tokio::test]
#[ignore]
async fn test_authorization_code_pkce() {
    let pool = setup_test_pool().await;
    let repo = OAuthRepository::new(pool);

    let code = "test_pkce_code";
    let client_id = "test_client_pkce";
    let user_id = Uuid::new_v4().to_string();
    let redirect_uri = "http://localhost:3000/callback";

    use sha2::{Digest, Sha256};
    let verifier = "test_verifier_string_that_is_long_enough_for_pkce";
    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let challenge =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());

    repo.store_authorization_code(
        code,
        client_id,
        &user_id,
        redirect_uri,
        "openid",
        Some(&challenge),
        Some("S256"),
    )
    .await
    .expect("Failed to store PKCE code");

    let validation = repo
        .validate_authorization_code(code, client_id, Some(redirect_uri), Some(verifier))
        .await
        .expect("Failed to validate PKCE code");

    assert_eq!(validation.0, user_id);

    let invalid_verifier_result = repo
        .validate_authorization_code(code, client_id, Some(redirect_uri), Some("wrong_verifier"))
        .await;

    assert!(invalid_verifier_result.is_err(), "Invalid verifier should fail");
}

#[tokio::test]
#[ignore]
async fn test_refresh_token_lifecycle() {
    let pool = setup_test_pool().await;
    let repo = OAuthRepository::new(pool);

    let token_id = "test_refresh_token_123";
    let client_id = "test_client_refresh";
    let user_id = Uuid::new_v4().to_string();
    let scopes = "openid profile";
    let expires_at = (Utc::now().timestamp() + 7 * 24 * 60 * 60) as i64;

    repo.store_refresh_token(token_id, client_id, &user_id, scopes, expires_at)
        .await
        .expect("Failed to store refresh token");

    let (returned_user_id, returned_scope) = repo
        .validate_refresh_token(token_id, client_id)
        .await
        .expect("Failed to validate refresh token");

    assert_eq!(returned_user_id, user_id);
    assert_eq!(returned_scope, scopes);

    let (user_from_consume, scope_from_consume) = repo
        .consume_refresh_token(token_id, client_id)
        .await
        .expect("Failed to consume refresh token");

    assert_eq!(user_from_consume, user_id);
    assert_eq!(scope_from_consume, scopes);

    let validation_after_consume = repo
        .validate_refresh_token(token_id, client_id)
        .await;

    assert!(validation_after_consume.is_err(), "Consumed token should not validate");
}

#[tokio::test]
#[ignore]
async fn test_refresh_token_expiration() {
    let pool = setup_test_pool().await;
    let repo = OAuthRepository::new(pool);

    let token_id = "test_expired_token";
    let client_id = "test_client_expired";
    let user_id = Uuid::new_v4().to_string();
    let scopes = "openid";
    let expires_at = (Utc::now().timestamp() - 1) as i64;

    repo.store_refresh_token(token_id, client_id, &user_id, scopes, expires_at)
        .await
        .expect("Failed to store expired token");

    let validation = repo
        .validate_refresh_token(token_id, client_id)
        .await;

    assert!(validation.is_err(), "Expired token should not validate");
}

#[tokio::test]
#[ignore]
async fn test_refresh_token_revocation() {
    let pool = setup_test_pool().await;
    let repo = OAuthRepository::new(pool);

    let token_id = "test_revoke_token";
    let client_id = "test_client_revoke";
    let user_id = Uuid::new_v4().to_string();
    let scopes = "openid";
    let expires_at = (Utc::now().timestamp() + 7 * 24 * 60 * 60) as i64;

    repo.store_refresh_token(token_id, client_id, &user_id, scopes, expires_at)
        .await
        .expect("Failed to store token");

    let revoked = repo
        .revoke_refresh_token(token_id)
        .await
        .expect("Failed to revoke token");

    assert!(revoked, "Token should have been revoked");

    let validation = repo
        .validate_refresh_token(token_id, client_id)
        .await;

    assert!(validation.is_err(), "Revoked token should not validate");
}
