//! Federated-identity link/unlink for externally-issued identities.
//!
//! One `(issuer, external_sub)` pair maps to one local `users` row. Inbound
//! chat platforms resolve senders through this table (core's messaging
//! pipeline calls `find_or_create_federated` against it), so attaching a pair
//! to an existing account is how an operator says "this Slack account is that
//! user" — and how the sender inherits that user's roles.
//!
//! Linking never steals: a pair already owned by another user is reported, not
//! reassigned. Unlink is per-issuer, so detaching Slack leaves any other
//! federated identity intact.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// Outcome of an explicit profile-driven link attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    AlreadyLinkedElsewhere,
}

async fn find_mapping(
    pool: &PgPool,
    issuer: &str,
    external_sub: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT user_id FROM federated_identities WHERE issuer = $1 AND external_sub = $2",
        issuer,
        external_sub
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| UserId::new(r.user_id)))
}

// Why: Attach an external identity to `user_id`. Idempotent when the pair
// already points at this user; refuses to steal a mapping owned by another.
pub async fn link_identity_to_user(
    pool: &PgPool,
    issuer: &str,
    external_sub: &str,
    user_id: &UserId,
) -> Result<LinkOutcome, sqlx::Error> {
    let inserted = sqlx::query!(
        "INSERT INTO federated_identities (issuer, external_sub, user_id) \
         VALUES ($1, $2, $3) ON CONFLICT (issuer, external_sub) DO NOTHING",
        issuer,
        external_sub,
        user_id.as_str()
    )
    .execute(pool)
    .await?
    .rows_affected();
    if inserted > 0 {
        return Ok(LinkOutcome::Linked);
    }
    match find_mapping(pool, issuer, external_sub).await? {
        Some(owner) if owner == *user_id => Ok(LinkOutcome::Linked),
        _ => Ok(LinkOutcome::AlreadyLinkedElsewhere),
    }
}

pub async fn delete_federated_identities_for_issuer(
    pool: &PgPool,
    user_id: &UserId,
    issuer: &str,
) -> Result<u64, sqlx::Error> {
    let deleted = sqlx::query!(
        "DELETE FROM federated_identities WHERE user_id = $1 AND issuer = $2",
        user_id.as_str(),
        issuer
    )
    .execute(pool)
    .await?
    .rows_affected();
    Ok(deleted)
}
