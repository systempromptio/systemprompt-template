//! The write half of federated resolution: just-in-time provisioning and the
//! per-login role projection. Split from [`super::federated`] so the read
//! ladder there stays readable; nothing here is called from outside it.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

use super::federated::{FederatedClaims, ResolvedFederatedUser};

// Why: `name` is set to the email to sidestep the `users.name` uniqueness
// constraint; `display_name` carries the human-friendly form.
pub(super) async fn create_federated(
    pool: &PgPool,
    claims: &FederatedClaims<'_>,
) -> Result<ResolvedFederatedUser, MarketplaceError> {
    let FederatedClaims {
        issuer,
        external_sub,
        email,
        display_name,
        roles,
    } = *claims;

    let user_id = uuid::Uuid::new_v4().to_string();
    // Why: an IdP that maps no roles still provisions a plain user; an empty
    // role list would be an account that can reach nothing.
    let roles = if roles.is_empty() {
        vec!["user".to_owned()]
    } else {
        roles.to_vec()
    };
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"
        INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
        VALUES ($1, $2, $3, $4, 'active', true, $5)
        "#,
        &user_id,
        email,
        email,
        display_name,
        &roles,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO federated_identities (issuer, external_sub, user_id) VALUES ($1, $2, $3)",
        issuer,
        external_sub,
        &user_id,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT DO NOTHING",
        &user_id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(ResolvedFederatedUser {
        user_id: UserId::new(user_id),
        email: email.to_owned(),
        display_name: display_name.to_owned(),
        roles,
        lost_admin: false,
    })
}

pub(super) struct ProjectedRoles {
    pub roles: Vec<String>,
    // Why: the JWT bakes permissions in at issue time, so a write-tier role
    // the directory has taken away still rides in every token minted before
    // this login. The caller uses this to tear those down.
    pub lost_admin: bool,
}

// Why: The IdP is the source of truth for the *directory* half of the role
// set, so a returning login rewrites it — but the effective set is that union
// the manual grants an admin made in the dashboard, which a sign-in must not
// wipe. Skipped when the claim carries nothing: an operator-created account
// signing in through an IdP that maps no roles (evaluation instances) keeps
// its CLI-granted roles rather than losing them.
pub(super) async fn project_roles(
    pool: &PgPool,
    user_id: &UserId,
    current: Vec<String>,
    mapped: &[String],
) -> Result<ProjectedRoles, sqlx::Error> {
    if mapped.is_empty() {
        return Ok(ProjectedRoles {
            roles: current,
            lost_admin: false,
        });
    }
    let roles = super::roles::recompute_roles(pool, user_id, Some(mapped)).await?;
    Ok(ProjectedRoles {
        lost_admin: has_manage(&current) && !has_manage(&roles),
        roles,
    })
}

// Why: any role in the write tier, not `admin` alone. `platform_admin` mints
// the same `Permission::Admin` into a token, so a directory that drops it has
// to reach the tokens issued while it was held, exactly as a dropped `admin`
// does.
fn has_manage(roles: &[String]) -> bool {
    crate::types::role::has_any(roles, crate::types::role::ROLES_MANAGE)
}
