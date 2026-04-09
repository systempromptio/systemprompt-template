use sqlx::PgPool;

pub async fn count_recent_setup_tokens(pool: &PgPool, email: &str) -> i64 {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM webauthn_setup_tokens
         WHERE user_id IN (SELECT id FROM users WHERE email = $1)
         AND created_at > NOW() - INTERVAL '15 minutes'",
        email,
    )
    .fetch_one(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(0)
}

pub async fn insert_setup_token(
    pool: &PgPool,
    token_id: &str,
    user_id: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO webauthn_setup_tokens (id, user_id, token_hash, purpose, expires_at)
         VALUES ($1, $2, $3, 'credential_link', NOW() + INTERVAL '15 minutes')",
        token_id,
        user_id,
        token_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}
