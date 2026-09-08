//! Bridge API key issue, hashing, and verification.

use chrono::{DateTime, Utc};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use uuid::Uuid;

use super::error::{BridgeRepoError, Result};

pub const API_KEY_PREFIX: &str = "sp-live-";
const SECRET_BYTES: usize = 32;
const PREFIX_ID_BYTES: usize = 6;

#[derive(Debug, sqlx::FromRow)]
pub struct ApiKeyRow {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct IssuedApiKey {
    pub id: String,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub created_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn issue_api_key(
    pool: &PgPool,
    user_id: &UserId,
    name: &str,
    expires_at: Option<DateTime<Utc>>,
) -> Result<IssuedApiKey> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(BridgeRepoError::Validation(
            "PAT name must not be empty".into(),
        ));
    }
    let id = format!("ak_{}", Uuid::new_v4().simple());
    let (secret, key_prefix, key_hash) = generate_secret();

    let row = sqlx::query!(
        r#"
        INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING created_at, expires_at
        "#,
        id,
        user_id.as_str(),
        trimmed,
        key_prefix,
        key_hash,
        expires_at,
    )
    .fetch_one(pool)
    .await?;

    Ok(IssuedApiKey {
        id,
        name: trimmed.to_owned(),
        key_prefix,
        secret,
        created_at: Some(row.created_at),
        expires_at: row.expires_at,
    })
}

#[derive(Debug)]
pub struct EnrolledDevice {
    pub id: String,
    pub user_id: UserId,
    pub name: String,
    pub key_prefix: String,
    pub secret: String,
    pub platform: String,
    pub hostname: String,
    pub created_at: Option<DateTime<Utc>>,
    pub enrolled_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct EnrollDeviceParams<'a> {
    pub name: &'a str,
    pub platform: &'a str,
    pub hostname: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
}

pub async fn enroll_device(
    pool: &PgPool,
    user_id: &UserId,
    params: EnrollDeviceParams<'_>,
) -> Result<EnrolledDevice> {
    let EnrollDeviceParams {
        name,
        platform,
        hostname,
        expires_at,
    } = params;
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(BridgeRepoError::Validation(
            "Device name must not be empty".into(),
        ));
    }
    let platform_norm = platform.trim().to_lowercase();
    if !matches!(platform_norm.as_str(), "macos" | "windows" | "linux") {
        return Err(BridgeRepoError::Validation(
            "Platform must be one of macos, windows, linux".into(),
        ));
    }
    let hostname_norm = hostname.trim().to_owned();

    let id = format!("ak_{}", Uuid::new_v4().simple());
    let (secret, key_prefix, key_hash) = generate_secret();

    let mut tx = pool.begin().await?;

    let key_row = sqlx::query!(
        r#"
        INSERT INTO user_api_keys (id, user_id, name, key_prefix, key_hash, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING created_at, expires_at
        "#,
        id,
        user_id.as_str(),
        trimmed,
        key_prefix,
        key_hash,
        expires_at,
    )
    .fetch_one(&mut *tx)
    .await?;

    let link_row = sqlx::query!(
        r#"
        INSERT INTO device_app_links (device_id, user_id, app_platform, app_version, hostname)
        VALUES ($1, $2, $3, '', $4)
        RETURNING enrolled_at
        "#,
        id,
        user_id.as_str(),
        platform_norm,
        hostname_norm,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(EnrolledDevice {
        id,
        user_id: user_id.clone(),
        name: trimmed.to_owned(),
        key_prefix,
        secret,
        platform: platform_norm,
        hostname: hostname_norm,
        created_at: Some(key_row.created_at),
        enrolled_at: link_row.enrolled_at,
        expires_at: key_row.expires_at,
    })
}

pub async fn list_api_keys_for_user(pool: &PgPool, user_id: &UserId) -> Result<Vec<ApiKeyRow>> {
    let rows = sqlx::query_as!(
        ApiKeyRow,
        r#"
        SELECT id, name, key_prefix, created_at, last_used_at, expires_at, revoked_at
        FROM user_api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn revoke_api_key(pool: &PgPool, user_id: &UserId, id: &str) -> Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE user_api_keys
        SET revoked_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL
        "#,
        id,
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

fn generate_secret() -> (String, String, String) {
    let mut raw = [0u8; SECRET_BYTES];
    rand::rng().fill_bytes(&mut raw);
    let encoded = hex::encode(raw);
    let key_prefix = format!("{API_KEY_PREFIX}{}", &encoded[..PREFIX_ID_BYTES * 2]);
    let secret = format!("{key_prefix}.{}", &encoded[PREFIX_ID_BYTES * 2..]);
    let key_hash = hash_secret(&secret);
    (secret, key_prefix, key_hash)
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}
