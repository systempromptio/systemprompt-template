//! Enrolment links issued and not yet claimed.
//!
//! A link is a ten-minute promise: a code was handed to somebody so their
//! machine could enrol itself. Only the hash is stored, so nothing here can
//! reveal the code — the listing exists to answer "who is mid-enrolment right
//! now", which is the question an operator asks when a colleague says the
//! bridge will not connect.
//!
//! Expired codes are listed alongside live ones rather than filtered away. A
//! code that ran out is the commonest reason an enrolment failed, and a page
//! that hid it would answer "nothing pending" to somebody staring at a
//! bridge that never linked. They are read per person, because a code has no
//! identity of its own: five rows for one name say nothing five times.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct PendingLinkRow {
    pub user_id: UserId,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_expired: bool,
}

#[derive(Debug, Clone)]
pub struct LinkUserRow {
    pub user_id: UserId,
    pub user_name: String,
    pub total: i64,
    pub pending: i64,
    pub newest_created_at: DateTime<Utc>,
    pub latest_expires_at: DateTime<Utc>,
}

pub async fn list_pending_link_users_paged(
    pool: &PgPool,
    sort: &str,
    dir: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<LinkUserRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        LinkUserRow,
        r#"SELECT c.user_id AS "user_id!: UserId", u.name AS "user_name!",
                  COUNT(*) AS "total!",
                  COUNT(*) FILTER (WHERE c.expires_at >= NOW()) AS "pending!",
                  MAX(c.created_at) AS "newest_created_at!",
                  MAX(c.expires_at) AS "latest_expires_at!"
             FROM bridge_exchange_codes c
             JOIN users u ON u.id = c.user_id
            WHERE c.consumed_at IS NULL
            GROUP BY c.user_id, u.name
            ORDER BY
              CASE WHEN $2::TEXT = 'desc' THEN
                CASE $1::TEXT WHEN 'expires' THEN EXTRACT(EPOCH FROM MAX(c.expires_at))
                        ELSE EXTRACT(EPOCH FROM MAX(c.created_at)) END
              END DESC NULLS LAST,
              CASE WHEN $2::TEXT = 'asc' THEN
                CASE $1::TEXT WHEN 'expires' THEN EXTRACT(EPOCH FROM MAX(c.expires_at))
                        ELSE EXTRACT(EPOCH FROM MAX(c.created_at)) END
              END ASC NULLS LAST,
              c.user_id
            LIMIT $3 OFFSET $4"#,
        sort,
        dir,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT user_id) AS "total!"
             FROM bridge_exchange_codes
            WHERE consumed_at IS NULL"#
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn list_pending_links_for_users(
    pool: &PgPool,
    user_ids: &[String],
) -> Result<Vec<PendingLinkRow>, sqlx::Error> {
    sqlx::query_as!(
        PendingLinkRow,
        r#"SELECT c.user_id AS "user_id!: UserId", c.created_at AS "created_at!",
                  c.expires_at AS "expires_at!",
                  (c.expires_at < NOW()) AS "is_expired!"
             FROM bridge_exchange_codes c
            WHERE c.consumed_at IS NULL
              AND c.user_id = ANY($1::TEXT[])
            ORDER BY c.user_id, c.created_at DESC, c.code_hash"#,
        user_ids,
    )
    .fetch_all(pool)
    .await
}
