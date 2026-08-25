//! Storage for a local user's Salesforce *Username* (userinfo
//! `preferred_username`).
//!
//! The Salesforce JWT-bearer grant matches its `sub` claim against the
//! Salesforce Username, which is not the login email (e.g.
//! `ed.aa…@agentforce.com` vs `ed@systemprompt.io`). The SSO callback captures
//! the Username here so the Hosted-MCP token accessor can mint a bearer as the
//! right user. Lives in the web-owned `salesforce_user_identities` side table
//! (schema/15), not the vendored `federated_identities` table.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// Why: Record (or refresh) the Salesforce Username for `user_id`. Idempotent: a
// repeat login overwrites the stored Username and bumps `updated_at`.
pub async fn upsert(pool: &PgPool, user_id: &UserId, sf_username: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO salesforce_user_identities (user_id, sf_username) \
         VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE \
         SET sf_username = EXCLUDED.sf_username, updated_at = CURRENT_TIMESTAMP",
        user_id.as_str(),
        sf_username
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: The source of truth for "who our users are" when provisioning an org:
// these are the accounts that completed a Salesforce SSO login, so these are
// the accounts `salesforce apply` assigns the MCP permission set to. Keeping
// the list here rather than in `org.yaml` keeps personal data out of the
// repository and out of a file that gets copied between orgs.
pub async fn list_salesforce_usernames(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT DISTINCT sf_username FROM salesforce_user_identities ORDER BY sf_username"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.sf_username).collect())
}

// Why: Remove the stored Salesforce Username for `user_id` (the profile
// "Disconnect" flow). Absent row is fine — the state is already what the
// caller asked for.
pub async fn delete(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM salesforce_user_identities WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: The Salesforce Username for `user_id`, or `None` if this user never
// completed a Salesforce SSO login (in which case the caller falls back to the
// email).
pub async fn find(pool: &PgPool, user_id: &UserId) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT sf_username FROM salesforce_user_identities WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.sf_username))
}

/// A local account and the Salesforce Username it is linked to.
#[derive(Debug, Clone)]
pub struct LinkedSalesforceIdentity {
    pub user_id: UserId,
    pub sf_username: String,
}

// Why: the deprovisioning reconciler needs the pair, not just the username —
// a deactivated Salesforce user has to map back to the local account being
// disabled. Only active local accounts are listed: re-disabling a disabled
// account every sweep would churn the audit log with no state change.
pub async fn list_linked_identities(
    pool: &PgPool,
) -> Result<Vec<LinkedSalesforceIdentity>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT s.user_id, s.sf_username
         FROM salesforce_user_identities s
         JOIN users u ON u.id = s.user_id
         WHERE u.status = 'active'
         ORDER BY s.sf_username"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| LinkedSalesforceIdentity {
            user_id: UserId::new(r.user_id),
            sf_username: r.sf_username,
        })
        .collect())
}
