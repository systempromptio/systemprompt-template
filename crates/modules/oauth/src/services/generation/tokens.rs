use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::models::JwtClaims;
use systemprompt_identifiers::SessionId;
use systemprompt_models::auth::{
    AuthenticatedUser, JwtAudience, Permission, RateLimitTier, TokenType, UserType,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    pub permissions: Vec<Permission>,
    pub audience: Vec<JwtAudience>,
    pub expires_in_hours: Option<i64>,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            permissions: vec![Permission::User],
            audience: JwtAudience::standard(),
            expires_in_hours: Some(24),
        }
    }
}

pub fn generate_secure_token(prefix: &str) -> String {
    let mut rng = thread_rng();
    let token: String = (0..32)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect();

    format!("{prefix}_{token}")
}

pub fn generate_jwt(
    user: &AuthenticatedUser,
    config: JwtConfig,
    jti: String,
    session_id: &SessionId,
    jwt_secret: &str,
) -> Result<String> {
    let expires_in_hours = config.expires_in_hours.unwrap_or(24);

    if expires_in_hours <= 0 || expires_in_hours > 8760 {
        return Err(anyhow!(
            "Invalid token expiry: {expires_in_hours} hours. Must be between 1 and 8760 (1 year)"
        ));
    }

    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expires_in_hours))
        .ok_or_else(|| anyhow!("Failed to calculate token expiration"))?
        .timestamp();

    let now = Utc::now().timestamp();
    let user_type = user.user_type();

    let claims = JwtClaims {
        // Standard JWT Claims (RFC 7519)
        sub: user.id.to_string(),
        iat: now,
        exp: expiration,
        iss: "systemprompt-os".to_string(),
        aud: config.audience.clone(),
        jti,

        // OAuth 2.0 Claims
        scope: config.permissions,

        // SystemPrompt User Claims
        username: user.username.clone(),
        email: user.email_or_default(),
        user_type,

        // Enhanced Security Claims
        client_id: None,
        token_type: TokenType::Bearer,
        auth_time: now,
        session_id: Some(session_id.to_string()),

        // Rate Limiting & Security Metadata
        rate_limit_tier: Some(user_type.rate_tier()),
    };

    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok(token)
}

pub fn generate_client_secret() -> String {
    let mut rng = thread_rng();
    let secret: String = (0..64)
        .map(|_| rng.sample(Alphanumeric))
        .map(char::from)
        .collect();

    format!("secret_{secret}")
}

pub fn generate_access_token_jti() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn hash_client_secret(secret: &str) -> Result<String> {
    use bcrypt::{hash, DEFAULT_COST};
    Ok(hash(secret, DEFAULT_COST)?)
}

pub fn verify_client_secret(secret: &str, hash: &str) -> Result<bool> {
    use bcrypt::verify;
    Ok(verify(secret, hash)?)
}

pub fn generate_anonymous_jwt(
    user_id: &str,
    session_id: &str,
    client_id: &systemprompt_identifiers::ClientId,
    jwt_secret: &str,
) -> Result<String> {
    let expires_in_hours =
        systemprompt_core_system::Config::global().jwt_access_token_expiration / 3600;
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(expires_in_hours))
        .ok_or_else(|| anyhow!("Failed to calculate token expiration"))?
        .timestamp();

    let now = Utc::now().timestamp();

    let claims = JwtClaims {
        sub: user_id.to_string(),
        iat: now,
        exp: expiration,
        iss: "systemprompt-os".to_string(),
        aud: JwtAudience::standard(),
        jti: uuid::Uuid::new_v4().to_string(),
        scope: vec![Permission::Anonymous],
        username: user_id.to_string(),
        email: user_id.to_string(),
        user_type: UserType::Anon,
        client_id: Some(client_id.as_str().to_string()),
        token_type: TokenType::Bearer,
        auth_time: now,
        session_id: Some(session_id.to_string()),
        rate_limit_tier: Some(RateLimitTier::Anon),
    };

    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )?;

    Ok(token)
}
