//! Server-owned connection metadata, generations and device-visible revisions.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct ProviderConnection {
    pub provider: String,
    pub status: String,
    pub auth_method: Option<String>,
    pub account_id: Option<String>,
    pub account_name: Option<String>,
    pub resource_id: Option<String>,
    pub resource_name: Option<String>,
    pub error_code: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub generation: i64,
    pub revision: i64,
}

pub async fn list_accounts(
    pool: &PgPool,
    user: &UserId,
) -> Result<Vec<ProviderConnection>, sqlx::Error> {
    sqlx::query_as!(
        ProviderConnection,
        "SELECT provider, status, auth_method, account_id, account_name,
        resource_id, resource_name, error_code, verified_at, generation, revision
        FROM mcp_connector_accounts WHERE user_id = $1 ORDER BY provider",
        user.as_str()
    )
    .fetch_all(pool)
    .await
}

// Why: All mutations take this lock before touching credentials, including
// disconnect and callbacks. A callback cannot resurrect a connection
// disconnected in flight.
pub async fn get_locked_account(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    provider: &str,
) -> Result<ProviderConnection, sqlx::Error> {
    sqlx::query!(
        "INSERT INTO mcp_connector_accounts (user_id, provider) VALUES ($1, $2)
        ON CONFLICT DO NOTHING",
        user.as_str(),
        provider
    )
    .execute(&mut **tx)
    .await?;
    sqlx::query_as!(
        ProviderConnection,
        "SELECT provider, status, auth_method, account_id, account_name,
        resource_id, resource_name, error_code, verified_at, generation, revision
        FROM mcp_connector_accounts WHERE user_id = $1 AND provider = $2 FOR UPDATE",
        user.as_str(),
        provider
    )
    .fetch_one(&mut **tx)
    .await
}

pub async fn update_account(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    row: &ProviderConnection,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE mcp_connector_accounts SET status = $3, auth_method = $4,
        account_id = $5, account_name = $6, resource_id = $7, resource_name = $8,
        error_code = $9, verified_at = $10, generation = $11,
        revision = nextval('mcp_connector_revision') WHERE user_id = $1 AND provider = $2",
        user.as_str(),
        row.provider,
        row.status,
        row.auth_method,
        row.account_id,
        row.account_name,
        row.resource_id,
        row.resource_name,
        row.error_code,
        row.verified_at,
        row.generation
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub async fn delete_pending(
    tx: &mut Transaction<'_, Postgres>,
    user: &UserId,
    provider: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM mcp_connector_oauth_states WHERE user_id = $1 AND provider = $2",
        user.as_str(),
        provider
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// Why: Session revocation is checked against the primary database on every
// action; UI polling and JWT expiry are never the revocation enforcement
// mechanism.
pub async fn is_live_session(
    pool: &PgPool,
    user: &UserId,
    session: &str,
) -> Result<bool, sqlx::Error> {
    // Why: ended_at tracks analytics inactivity; only revocation and expiry end
    // authorization.
    sqlx::query_scalar!(
        r#"SELECT EXISTS(SELECT 1 FROM user_sessions s JOIN users u ON u.id = s.user_id
        WHERE s.session_id = $1 AND s.user_id = $2 AND s.revoked_at IS NULL
        AND s.expires_at > NOW() AND u.status = 'active') AS "live!""#,
        session,
        user.as_str()
    )
    .fetch_one(pool)
    .await
}
