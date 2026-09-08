//! Personal access tokens, read for the whole estate rather than one account.
//!
//! Only the prefix is stored in a form that can be shown, which is the point:
//! the listing identifies a credential well enough to revoke it and not well
//! enough to use it. A row's state is derived from two nullable timestamps —
//! revoked beats expired, because a revoked token that later passes its expiry
//! is still revoked and reporting it otherwise would suggest it lapsed on its
//! own.
//!
//! The page is read per person: a page of holders with their counts, then the
//! tokens of the holders on that page. One person with four tokens is one
//! row that opens, not four rows that repeat a name.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct FleetApiKeyRow {
    pub id: String,
    pub user_id: UserId,
    pub name: String,
    pub key_prefix: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyUserRow {
    pub user_id: UserId,
    pub user_name: String,
    pub total: i64,
    pub active: i64,
    pub newest_created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub next_expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy)]
pub struct CredentialQuery<'a> {
    // Why: `all`, `active` or `revoked`. Matched in SQL rather than branched
    // in Rust so the count and the page come from one predicate.
    pub state: &'a str,
    pub sort: &'a str,
    pub dir: &'a str,
    pub limit: i64,
    pub offset: i64,
}

pub async fn list_api_key_users_paged(
    pool: &PgPool,
    query: CredentialQuery<'_>,
) -> Result<(Vec<ApiKeyUserRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        ApiKeyUserRow,
        r#"SELECT k.user_id AS "user_id!: UserId", u.name AS "user_name!",
                  COUNT(*) AS "total!",
                  COUNT(*) FILTER (WHERE k.revoked_at IS NULL
                                     AND (k.expires_at IS NULL OR k.expires_at > NOW()))
                      AS "active!",
                  MAX(k.created_at) AS "newest_created_at!",
                  MAX(k.last_used_at) AS "last_used_at",
                  MIN(k.expires_at) FILTER (WHERE k.revoked_at IS NULL AND k.expires_at > NOW())
                      AS "next_expires_at"
             FROM user_api_keys k
             JOIN users u ON u.id = k.user_id
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND k.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND k.revoked_at IS NOT NULL)
            GROUP BY k.user_id, u.name
            ORDER BY
              CASE WHEN $3::TEXT = 'desc' THEN
                CASE $2::TEXT WHEN 'used' THEN EXTRACT(EPOCH FROM MAX(k.last_used_at))
                        WHEN 'expires' THEN EXTRACT(EPOCH FROM MIN(k.expires_at))
                        ELSE EXTRACT(EPOCH FROM MAX(k.created_at)) END
              END DESC NULLS LAST,
              CASE WHEN $3::TEXT = 'asc' THEN
                CASE $2::TEXT WHEN 'used' THEN EXTRACT(EPOCH FROM MAX(k.last_used_at))
                        WHEN 'expires' THEN EXTRACT(EPOCH FROM MIN(k.expires_at))
                        ELSE EXTRACT(EPOCH FROM MAX(k.created_at)) END
              END ASC NULLS LAST,
              k.user_id
            LIMIT $4 OFFSET $5"#,
        query.state,
        query.sort,
        query.dir,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT k.user_id) AS "total!"
             FROM user_api_keys k
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND k.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND k.revoked_at IS NOT NULL)"#,
        query.state,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn list_api_keys_for_users(
    pool: &PgPool,
    user_ids: &[String],
    state: &str,
) -> Result<Vec<FleetApiKeyRow>, sqlx::Error> {
    sqlx::query_as!(
        FleetApiKeyRow,
        r#"SELECT k.id AS "id!", k.user_id AS "user_id!: UserId", k.name AS "name!",
                  k.key_prefix AS "key_prefix!", k.created_at AS "created_at!",
                  k.last_used_at, k.expires_at, k.revoked_at
             FROM user_api_keys k
            WHERE k.user_id = ANY($1::TEXT[])
              AND (($2::TEXT = 'all')
                   OR ($2::TEXT = 'active' AND k.revoked_at IS NULL)
                   OR ($2::TEXT = 'revoked' AND k.revoked_at IS NOT NULL))
            ORDER BY k.user_id, k.created_at DESC, k.id"#,
        user_ids,
        state,
    )
    .fetch_all(pool)
    .await
}

// Why: an admin revoking someone else's token is a different act from a user
// revoking their own, and it is a different statement: there is no `user_id`
// in the predicate. The route that reaches it is gated on the admin roles,
// which is where that authority is granted and the only place it is.
pub async fn revoke_any_api_key(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE user_api_keys
              SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND revoked_at IS NULL"#,
        id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
