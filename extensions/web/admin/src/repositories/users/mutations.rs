//! User create, update, and delete.
//!
//! Creation is one of the two doors a seat is minted through (the other is SSO
//! just-in-time provisioning in [`super::federated`]). Both resolve the
//! organization the same way — from the email's domain — so which door a user
//! arrives through cannot change whose contract they land on, and neither door
//! can be the one that forgot to check the seat limit.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use crate::repositories::organizations;
use crate::types::{CreateUserRequest, UpdateUserRequest, UserSummary};

/// The created (or adopted) row plus the organization the email domain
/// resolved to — the handler needs the org to mint a credential-bootstrap
/// invite without re-deriving it and risking divergence.
#[derive(Debug)]
pub struct CreatedUser {
    pub summary: UserSummary,
    pub org_id: Option<String>,
}

pub async fn create_user(
    pool: &PgPool,
    req: &CreateUserRequest,
) -> Result<CreatedUser, MarketplaceError> {
    // Why: resolved before the insert so a full plan rejects the request
    // rather than leaving behind a user who exists but is entitled to nothing.
    // An unclaimed domain is not an error — that user is not on a customer
    // contract and consumes nobody's seat.
    let org_id = organizations::crud::find_organization_for_email(pool, req.email.as_str()).await?;
    if let Some(org_id) = org_id.as_deref()
        && !is_existing_member(pool, &req.user_id, org_id).await?
    {
        organizations::seats::assert_seat_available(pool, org_id).await?;
    }

    let user_id_str = req.user_id.as_str().to_owned();
    let status = req.status.clone().unwrap_or_else(|| "active".to_owned());
    let username = req.email.as_str();
    let mut tx = pool.begin().await?;
    let summary = sqlx::query_as!(
        UserSummary,
        r#"
        INSERT INTO users (id, name, email, display_name, roles, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET
            display_name = COALESCE(EXCLUDED.display_name, users.display_name),
            roles = EXCLUDED.roles,
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING
            id AS "user_id!",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            created_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        &user_id_str,
        username,
        req.email.as_str(),
        &req.display_name,
        &req.roles,
        &status,
    )
    .fetch_one(&mut *tx)
    .await?;

    if let Some(department) = req
        .department
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        sqlx::query!(
            r#"
            INSERT INTO user_profile_ext (user_id, department)
            VALUES ($1, $2)
            ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department
            "#,
            summary.user_id.as_str(),
            department,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;

    if let Some(org_id) = org_id.as_deref() {
        organizations::crud::set_membership(pool, &summary.user_id, org_id, "member").await?;
    }

    Ok(CreatedUser { summary, org_id })
}

async fn is_existing_member(
    pool: &PgPool,
    user_id: &UserId,
    org_id: &str,
) -> Result<bool, MarketplaceError> {
    let found = sqlx::query_scalar!(
        "SELECT 1 AS present FROM organization_members WHERE user_id = $1 AND org_id = $2",
        user_id.as_str(),
        org_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(found.is_some())
}

pub async fn update_user(
    pool: &PgPool,
    user_id: &UserId,
    req: &UpdateUserRequest,
) -> Result<Option<UserSummary>, sqlx::Error> {
    let status = req.is_active.map(|active| {
        if active {
            "active".to_owned()
        } else {
            "inactive".to_owned()
        }
    });
    let set_email_verified = req.is_active == Some(true);
    let roles_update: Option<&[String]> = req.roles.as_deref();
    let mut tx = pool.begin().await?;

    let summary = sqlx::query_as!(
        UserSummary,
        r#"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            email = COALESCE($3, email),
            roles = COALESCE($4, roles),
            status = COALESCE($5, status),
            email_verified = CASE WHEN $6 THEN true ELSE email_verified END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id AS "user_id!",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            updated_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        user_id.as_str(),
        req.display_name.as_deref(),
        req.email.as_deref(),
        roles_update,
        status.as_deref(),
        set_email_verified,
    )
    .fetch_optional(&mut *tx)
    .await?;

    if summary.is_some()
        && let Some(department) = req.department.as_deref()
    {
        sqlx::query!(
            r#"
                INSERT INTO user_profile_ext (user_id, department)
                VALUES ($1, $2)
                ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department
                "#,
            user_id.as_str(),
            department,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(summary)
}

pub async fn delete_user(pool: &PgPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id.as_str())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
