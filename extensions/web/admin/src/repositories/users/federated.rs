//! Federated-identity resolution for external SSO (ADFS).
//!
//! Bridges an externally-issued identity (`issuer`, `external_sub`) to a local
//! `users` row, honouring the "merge by verified email" rule that core's own
//! `find_or_create_federated` deliberately omits.
//!
//! Resolution order (the first match wins):
//! 1. **Existing mapping** — the `(issuer, external_sub)` pair already points
//!    at a user (a returning SSO login).
//! 2. **Email link** — an active local account already owns this email. We
//!    attach the federated identity to it instead of minting a duplicate. This
//!    is the account *merge*. The caller MUST have verified `email_verified`
//!    and an allow-listed domain before reaching this path — linking an
//!    unverified address would let a hostile `IdP` claim arbitrary accounts.
//! 3. **Create** — no mapping and no local account: provision a fresh user, the
//!    mapping, and its profile row, in a single transaction.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use super::federated_provision as provision;

/// Outcome of [`resolve_federated_user`]: a local user the caller can mint a
/// session for.
#[derive(Debug, Clone)]
pub struct ResolvedFederatedUser {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
    // Why: the directory took `admin` away on this login. The caller must
    // revoke what was minted while the user still held it.
    pub lost_admin: bool,
}

/// The verified external-identity claims carried into resolution. The caller
/// must have already enforced the allow-listed-domain + group gate.
#[derive(Debug, Clone, Copy)]
pub struct FederatedClaims<'a> {
    pub issuer: &'a str,
    pub external_sub: &'a str,
    pub email: &'a str,
    pub display_name: &'a str,
    // Why: the roles the IdP's group claim maps to. Written on create and
    // re-projected onto `users.roles` on every returning login (replace, not
    // union), so the directory — not a stale row — decides what a user holds.
    pub roles: &'a [String],
}

struct LocalUser {
    id: UserId,
    display_name: String,
    roles: Vec<String>,
}

async fn find_mapping(
    pool: &PgPool,
    issuer: &str,
    external_sub: &str,
) -> Result<Option<UserId>, sqlx::Error> {
    let row = sqlx::query!(
        "UPDATE federated_identities SET last_seen_at = CURRENT_TIMESTAMP \
         WHERE issuer = $1 AND external_sub = $2 RETURNING user_id",
        issuer,
        external_sub
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| UserId::new(r.user_id)))
}

async fn find_active_user_by_email(
    pool: &PgPool,
    email: &str,
) -> Result<Option<LocalUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id AS "id: UserId", COALESCE(display_name, name) AS "display_name!", roles AS "roles!: Vec<String>"
        FROM users
        WHERE LOWER(email) = LOWER($1) AND status = 'active'
        "#,
        email
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| LocalUser {
        id: r.id,
        display_name: r.display_name,
        roles: r.roles,
    }))
}

async fn load_user(pool: &PgPool, user_id: &UserId) -> Result<Option<LocalUser>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT id AS "id: UserId", COALESCE(display_name, name) AS "display_name!", roles AS "roles!: Vec<String>"
        FROM users WHERE id = $1
        "#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| LocalUser {
        id: r.id,
        display_name: r.display_name,
        roles: r.roles,
    }))
}

async fn link_existing(
    pool: &PgPool,
    issuer: &str,
    external_sub: &str,
    user_id: &UserId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO federated_identities (issuer, external_sub, user_id) \
         VALUES ($1, $2, $3) ON CONFLICT (issuer, external_sub) DO NOTHING",
        issuer,
        external_sub,
        user_id.as_str()
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Outcome of an explicit profile-driven link attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkOutcome {
    Linked,
    AlreadyLinkedElsewhere,
}

// Why: Attach an external identity to `user_id` (the profile "Connect" flow).
// Idempotent when the pair already points at this user; refuses to steal a
// mapping owned by another user.
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

// Why: `email` / `display_name` come from the verified `IdP` claims. The caller
// is responsible for the upstream gate (`email_verified == true` and an
// allow-listed domain) before invoking this.
//
// When `auto_provision` is `false` and the identity matches neither an
// existing mapping nor an active local account, returns `Ok(None)` — the
// caller should surface "this account must be created by an admin first"
// rather than minting a session.
pub async fn resolve_federated_user(
    pool: &PgPool,
    claims: &FederatedClaims<'_>,
    auto_provision: bool,
) -> Result<Option<ResolvedFederatedUser>, MarketplaceError> {
    let FederatedClaims {
        issuer,
        external_sub,
        email,
        display_name: _,
        roles,
    } = *claims;
    if let Some(user_id) = find_mapping(pool, issuer, external_sub).await?
        && let Some(user) = load_user(pool, &user_id).await?
    {
        let projected = provision::project_roles(pool, &user.id, user.roles, roles).await?;
        return Ok(Some(ResolvedFederatedUser {
            user_id: user.id,
            email: email.to_owned(),
            display_name: user.display_name,
            roles: projected.roles,
            lost_admin: projected.lost_admin,
        }));
    }

    if let Some(user) = find_active_user_by_email(pool, email).await? {
        link_existing(pool, issuer, external_sub, &user.id).await?;
        let projected = provision::project_roles(pool, &user.id, user.roles, roles).await?;
        return Ok(Some(ResolvedFederatedUser {
            user_id: user.id,
            email: email.to_owned(),
            display_name: user.display_name,
            roles: projected.roles,
            lost_admin: projected.lost_admin,
        }));
    }

    if !auto_provision {
        return Ok(None);
    }

    provision::create_federated(pool, claims).await.map(Some)
}

/// One identity provider mapping, as the user-detail page lists them.
#[derive(Debug, Clone, serde::Serialize)]
pub struct LinkedIdentityRow {
    pub issuer: String,
    pub external_sub: String,
    pub linked_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

// Why: the Identity tab lists every provider that vouches for this account, not
// just the most recent one `find_identity_envelope` returns — an admin
// unlinking Slack needs to see that Slack is linked while ADFS also is.
pub async fn list_linked_identities(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<LinkedIdentityRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT issuer AS "issuer!", external_sub AS "external_sub!",
                  created_at, last_seen_at
             FROM federated_identities
            WHERE user_id = $1
            ORDER BY last_seen_at DESC NULLS LAST, issuer"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| LinkedIdentityRow {
            issuer: row.issuer,
            external_sub: row.external_sub,
            linked_at: row.created_at,
            last_seen_at: row.last_seen_at,
        })
        .collect())
}
