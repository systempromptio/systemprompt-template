//! Storage for a local user's Salesforce *Username*.
//!
//! The Salesforce JWT-bearer grant matches its `sub` claim against the
//! Salesforce Username, which is not the login email (e.g.
//! `ed.aa…@agentforce.com` vs `ed@systemprompt.io`). Salesforce SSO used to
//! capture it from the userinfo `preferred_username` claim at login; ADFS
//! replaced that login, so the mapping is now set administratively and this
//! table is the only place it exists.
//!
//! Lives in the web-owned `salesforce_user_identities` side table (schema/21),
//! not the vendored `federated_identities` table.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

// Why: idempotent — re-linking overwrites the stored Username and bumps
// `updated_at` rather than erroring, so correcting a typo is one call.
pub async fn upsert_identity(
    pool: &PgPool,
    user_id: &UserId,
    sf_username: &str,
) -> Result<(), sqlx::Error> {
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

// Why: absent row is fine — the state is already what the caller asked for.
pub async fn delete_identity(pool: &PgPool, user_id: &UserId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "DELETE FROM salesforce_user_identities WHERE user_id = $1",
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: `None` means this user was never linked. The accessor treats that as a
// clean "not linked" denial rather than attempting a mint that cannot succeed.
pub async fn find_username(pool: &PgPool, user_id: &UserId) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT sf_username FROM salesforce_user_identities WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.sf_username))
}

// Why: the authz dimension's only question. Separate from `find` so the gate
// never pulls a username it has no use for.
pub async fn is_salesforce_linked(pool: &PgPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    Ok(find_username(pool, user_id).await?.is_some())
}
