use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::PgPool;

pub async fn create_magic_link_token(
    pool: &PgPool,
    email: &str,
    ip_address: Option<&str>,
) -> Result<String, anyhow::Error> {
    let (raw_token, token_hash) = {
        let mut rng = rand::rng();
        let raw_bytes: [u8; 32] = rng.random();
        let raw_token = hex::encode(raw_bytes);
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());
        (raw_token, token_hash)
    };

    sqlx::query!(
        "INSERT INTO marketplace.magic_link_tokens (email, token_hash, expires_at, ip_address)
         VALUES ($1, $2, NOW() + INTERVAL '15 minutes', $3)",
        email,
        &token_hash,
        ip_address,
    )
    .execute(pool)
    .await?;

    Ok(raw_token)
}

pub async fn consume_magic_link_token(
    pool: &PgPool,
    raw_token: &str,
) -> Result<String, anyhow::Error> {
    let token_hash = {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        hex::encode(hasher.finalize())
    };

    let row = sqlx::query_scalar!(
        "UPDATE marketplace.magic_link_tokens
         SET used = true, used_at = NOW()
         WHERE token_hash = $1 AND used = false AND expires_at > NOW()
         RETURNING email",
        &token_hash,
    )
    .fetch_optional(pool)
    .await?;

    row.ok_or_else(|| anyhow::anyhow!("Invalid or expired magic link"))
}

pub async fn count_recent_tokens(pool: &PgPool, email: &str) -> Result<i64, anyhow::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM marketplace.magic_link_tokens
         WHERE email = $1 AND created_at > NOW() - INTERVAL '15 minutes'",
        email,
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn count_recent_tokens_by_ip(pool: &PgPool, ip: &str) -> Result<i64, anyhow::Error> {
    let count = sqlx::query_scalar!(
        "SELECT COUNT(*) FROM marketplace.magic_link_tokens
         WHERE ip_address = $1 AND created_at > NOW() - INTERVAL '15 minutes'",
        ip,
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0))
}

pub async fn user_exists_by_email(pool: &PgPool, email: &str) -> Result<bool, anyhow::Error> {
    let row = sqlx::query_scalar!(
        "SELECT 1::BIGINT FROM users WHERE email = $1 LIMIT 1",
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}
