//! Tearing down everything a user already holds when SSO stops vouching for
//! them.
//!
//! Active Directory is the source of truth, but it is only consulted at
//! sign-in: a cookie session and a linked bridge both outlive the assertion
//! that created them. Removing someone from their group, or closing their
//! account, has to reach back into what they were already given, or the
//! directory decides nothing until the credentials expire on their own.
//!
//! Everything here is an `UPDATE` setting `revoked_at`, never a `DELETE` — the
//! rows are the audit trail of what that person held and when it was taken
//! away. One transaction, so a partial teardown cannot leave a live bridge PAT
//! behind a dead session.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RevokedCounts {
    pub sessions: u64,
    pub api_keys: u64,
    pub device_certs: u64,
    pub exchange_codes: u64,
}

impl RevokedCounts {
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.sessions == 0
            && self.api_keys == 0
            && self.device_certs == 0
            && self.exchange_codes == 0
    }
}

// Why: the unconsumed exchange codes go too. A code minted moments before the
// group was pulled is a bridge link waiting to happen, and it is the one
// credential here that a revoked user could still redeem into a fresh PAT.
pub async fn revoke_user_access(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<RevokedCounts, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let uid = user_id.as_str();

    let sessions = sqlx::query!(
        "UPDATE user_sessions SET revoked_at = NOW(), ended_at = COALESCE(ended_at, NOW())
         WHERE user_id = $1 AND revoked_at IS NULL",
        uid,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let api_keys = sqlx::query!(
        "UPDATE user_api_keys SET revoked_at = CURRENT_TIMESTAMP
         WHERE user_id = $1 AND revoked_at IS NULL",
        uid,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let device_certs = sqlx::query!(
        "UPDATE user_device_certs SET revoked_at = CURRENT_TIMESTAMP
         WHERE user_id = $1 AND revoked_at IS NULL",
        uid,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    let exchange_codes = sqlx::query!(
        "UPDATE bridge_exchange_codes SET consumed_at = NOW()
         WHERE user_id = $1 AND consumed_at IS NULL AND expires_at > NOW()",
        uid,
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // Why: Revoke downstream grants in the same transaction as device sessions.
    // Advancing generations also invalidates consent callbacks already in flight.
    sqlx::query!(
        "UPDATE mcp_connector_accounts SET status = 'not_connected', auth_method = NULL,
        error_code = 'user_revoked', generation = generation + 1,
        revision = nextval('mcp_connector_revision') WHERE user_id = $1",
        uid
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM mcp_connector_credentials WHERE user_id = $1",
        uid
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "DELETE FROM mcp_connector_oauth_states WHERE user_id = $1",
        uid
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(RevokedCounts {
        sessions,
        api_keys,
        device_certs,
        exchange_codes,
    })
}

// Why: the group gate fails before an identity is resolved, so the only handle
// on the person being turned away is the address the assertion carried. An
// inactive row is skipped: `revoke_user_access` is about live credentials, and
// a closed account's were taken when it closed.
pub async fn revoke_access_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(UserId, RevokedCounts)>, sqlx::Error> {
    let Some(row) = sqlx::query!(
        r#"SELECT id AS "id: UserId" FROM users WHERE LOWER(email) = LOWER($1) AND status = 'active'"#,
        email
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    let counts = revoke_user_access(pool, &row.id).await?;
    Ok(Some((row.id, counts)))
}
