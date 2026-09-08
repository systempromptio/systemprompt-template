//! Device certificates: the long-lived key a bridge installation presents.
//!
//! A certificate outlives the session it was enrolled for, so revoking one is
//! the only act that actually removes a machine from the estate — a bridge
//! session merely stops beating. The fingerprint is shown in full because it
//! is the identifier an operator matches against the machine in front of them,
//! and it is not a secret. Read per person, like the tokens: a page of
//! holders, then the certificates of the holders on that page.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::pats::CredentialQuery;

#[derive(Debug, Clone)]
pub struct FleetCertRow {
    pub id: String,
    pub user_id: UserId,
    pub label: String,
    pub fingerprint: String,
    pub enrolled_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct CertUserRow {
    pub user_id: UserId,
    pub user_name: String,
    pub total: i64,
    pub active: i64,
    pub latest_enrolled_at: DateTime<Utc>,
}

pub async fn list_device_cert_users_paged(
    pool: &PgPool,
    query: CredentialQuery<'_>,
) -> Result<(Vec<CertUserRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        CertUserRow,
        r#"SELECT c.user_id AS "user_id!: UserId", u.name AS "user_name!",
                  COUNT(*) AS "total!",
                  COUNT(*) FILTER (WHERE c.revoked_at IS NULL) AS "active!",
                  MAX(c.enrolled_at) AS "latest_enrolled_at!"
             FROM user_device_certs c
             JOIN users u ON u.id = c.user_id
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND c.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND c.revoked_at IS NOT NULL)
            GROUP BY c.user_id, u.name
            ORDER BY
              CASE WHEN $2::TEXT = 'desc' THEN EXTRACT(EPOCH FROM MAX(c.enrolled_at)) END
                DESC NULLS LAST,
              CASE WHEN $2::TEXT = 'asc' THEN EXTRACT(EPOCH FROM MAX(c.enrolled_at)) END
                ASC NULLS LAST,
              c.user_id
            LIMIT $3 OFFSET $4"#,
        query.state,
        query.dir,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(DISTINCT c.user_id) AS "total!"
             FROM user_device_certs c
            WHERE ($1::TEXT = 'all')
               OR ($1::TEXT = 'active' AND c.revoked_at IS NULL)
               OR ($1::TEXT = 'revoked' AND c.revoked_at IS NOT NULL)"#,
        query.state,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn list_device_certs_for_users(
    pool: &PgPool,
    user_ids: &[String],
    state: &str,
) -> Result<Vec<FleetCertRow>, sqlx::Error> {
    sqlx::query_as!(
        FleetCertRow,
        r#"SELECT c.id AS "id!", c.user_id AS "user_id!: UserId", c.label AS "label!",
                  c.fingerprint AS "fingerprint!", c.enrolled_at AS "enrolled_at!",
                  c.revoked_at
             FROM user_device_certs c
            WHERE c.user_id = ANY($1::TEXT[])
              AND (($2::TEXT = 'all')
                   OR ($2::TEXT = 'active' AND c.revoked_at IS NULL)
                   OR ($2::TEXT = 'revoked' AND c.revoked_at IS NOT NULL))
            ORDER BY c.user_id, c.enrolled_at DESC, c.id"#,
        user_ids,
        state,
    )
    .fetch_all(pool)
    .await
}

pub async fn revoke_any_device_cert(pool: &PgPool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE user_device_certs
              SET revoked_at = CURRENT_TIMESTAMP
            WHERE id = $1 AND revoked_at IS NULL"#,
        id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
