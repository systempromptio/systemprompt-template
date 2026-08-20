//! Link-based user invites.
//!
//! An admin mints a token (stored only as a SHA-256 hash); the invitee opens
//! the link, which provisions their account through the passkey path with the
//! org, department, and roles recorded on the invite. The explicit invite is
//! the authorization, so acceptance bypasses the `email_allowed` domain gate
//! that self-serve registration enforces.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use crate::repositories::organizations;

#[derive(Debug, Clone)]
pub struct UserInvite {
    pub id: String,
    pub email: String,
    pub org_id: String,
    pub org_name: String,
    pub department: String,
    pub roles: Vec<String>,
    pub invited_by: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct PendingInviteRow {
    pub id: String,
    pub email: String,
    pub org_id: String,
    pub org_name: String,
    pub department: String,
    pub roles: Vec<String>,
    pub invited_by: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Copy)]
pub struct NewInvite<'a> {
    pub email: &'a str,
    pub token_hash: &'a str,
    pub org_id: &'a str,
    pub department: &'a str,
    pub roles: &'a [String],
    pub invited_by: &'a UserId,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

pub async fn insert_invite(
    pool: &PgPool,
    params: &NewInvite<'_>,
) -> Result<String, MarketplaceError> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO user_invites
            (id, token_hash, email, org_id, department, roles, invited_by, expires_at)
         VALUES ($1, $2, LOWER($3), $4, $5, $6, $7, $8)",
        &id,
        params.token_hash,
        params.email,
        params.org_id,
        params.department,
        params.roles,
        params.invited_by.as_str(),
        params.expires_at,
    )
    .execute(pool)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db) if db.is_unique_violation() => MarketplaceError::Conflict(
            "A pending invite for this email already exists — revoke it first.".to_owned(),
        ),
        _ => e.into(),
    })?;
    Ok(id)
}

/// Pending (not accepted, not revoked, not expired) invites, optionally
/// limited to one organization for org-admin callers.
pub async fn list_pending_invites(
    pool: &PgPool,
    org_id: Option<&str>,
) -> Result<Vec<PendingInviteRow>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT i.id AS "id!", i.email AS "email!", i.org_id AS "org_id!",
               o.name AS "org_name!", i.department AS "department!",
               i.roles AS "roles!: Vec<String>", i.invited_by AS "invited_by!",
               i.expires_at AS "expires_at!", i.created_at AS "created_at!"
        FROM user_invites i
        JOIN organizations o ON o.id = i.org_id
        WHERE i.accepted_at IS NULL AND i.revoked_at IS NULL
          AND i.expires_at > NOW()
          AND ($1::TEXT IS NULL OR i.org_id = $1)
        ORDER BY i.created_at DESC
        "#,
        org_id,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| PendingInviteRow {
            id: r.id,
            email: r.email,
            org_id: r.org_id,
            org_name: r.org_name,
            department: r.department,
            roles: r.roles,
            invited_by: r.invited_by,
            expires_at: r.expires_at,
            created_at: r.created_at,
        })
        .collect())
}

pub async fn find_valid_invite_by_hash(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<UserInvite>, MarketplaceError> {
    let row = sqlx::query!(
        r#"
        SELECT i.id AS "id!", i.email AS "email!", i.org_id AS "org_id!",
               o.name AS "org_name!", i.department AS "department!",
               i.roles AS "roles!: Vec<String>",
               i.invited_by AS "invited_by!", i.expires_at AS "expires_at!"
        FROM user_invites i
        JOIN organizations o ON o.id = i.org_id
        WHERE i.token_hash = $1
          AND i.accepted_at IS NULL AND i.revoked_at IS NULL AND i.expires_at > NOW()
        "#,
        token_hash,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| UserInvite {
        id: r.id,
        email: r.email,
        org_id: r.org_id,
        org_name: r.org_name,
        department: r.department,
        roles: r.roles,
        invited_by: r.invited_by,
        expires_at: r.expires_at,
    }))
}

/// Revoke a pending invite. `org_id` restricts the delete for org-admin
/// callers; `None` (platform admin) may revoke any. Returns whether a row
/// was revoked.
pub async fn revoke_invite(
    pool: &PgPool,
    invite_id: &str,
    org_id: Option<&str>,
) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "UPDATE user_invites SET revoked_at = NOW()
         WHERE id = $1 AND accepted_at IS NULL AND revoked_at IS NULL
           AND ($2::TEXT IS NULL OR org_id = $2)",
        invite_id,
        org_id,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Provision the invited user and mark the invite accepted, in one transaction.
///
/// Mirrors `passkey::insert_passkey_user`'s rules (`name = email`,
/// `email_verified` on the invite's authority) but takes the org, department,
/// and roles from the invite instead of the email domain.
///
/// An existing active account with this email is adopted rather than
/// recreated — the invite then just (re)assigns org, department, and roles.
pub async fn accept_invite_and_provision(
    pool: &PgPool,
    invite: &UserInvite,
) -> Result<UserId, MarketplaceError> {
    let already_member = sqlx::query_scalar!(
        "SELECT 1 AS present FROM organization_members m
         JOIN users u ON u.id = m.user_id
         WHERE LOWER(u.email) = LOWER($1) AND m.org_id = $2",
        invite.email,
        invite.org_id,
    )
    .fetch_optional(pool)
    .await?
    .is_some();
    if !already_member {
        organizations::seats::assert_seat_available(pool, &invite.org_id).await?;
    }

    let new_id = uuid::Uuid::new_v4().to_string();
    let mut tx = pool.begin().await?;

    let user_id = sqlx::query_scalar!(
        r#"
        INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
        VALUES ($1, LOWER($2), LOWER($2), $2, 'active', true, $3)
        ON CONFLICT (email) DO UPDATE SET
            status = 'active',
            roles = EXCLUDED.roles,
            updated_at = NOW()
        RETURNING id AS "id!: UserId"
        "#,
        &new_id,
        invite.email,
        &invite.roles,
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO organization_members (user_id, org_id, org_role)
         VALUES ($1, $2, 'member')
         ON CONFLICT (user_id) DO UPDATE
            SET org_id = EXCLUDED.org_id, org_role = EXCLUDED.org_role",
        user_id.as_str(),
        invite.org_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO user_profile_ext (user_id, department)
         VALUES ($1, $2)
         ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department",
        user_id.as_str(),
        invite.department,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "UPDATE user_invites SET accepted_at = NOW() WHERE id = $1",
        invite.id,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(user_id)
}
