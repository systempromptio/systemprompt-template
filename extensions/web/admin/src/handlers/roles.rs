//! Reading and setting a user's roles.
//!
//! Roles moved off `PUT /users/{id}` onto their own pair of routes because
//! they are the one field with a rule attached: only a platform admin may
//! move `platform_admin`, the last one cannot be demoted, and a role the
//! directory grants cannot be revoked here. A field on a general update
//! request could not express any of that.
//!
//! What is written is the manual half alone. The effective set on
//! `users.roles` is recomputed from it and the directory half, so a role AD
//! grants survives this edit and one AD later withdraws disappears without
//! anything here having to remember it.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{revocation, roles as repo};
use crate::types::role::{ROLES_MANAGE, authorize_role_change, has_any};
use crate::types::{SetUserRolesRequest, UserContext, UserRolesResponse};

pub(crate) async fn get_user_roles_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id);
    let roles = require_effective_roles(&pool, &user_id).await?;
    let manual_roles = repo::list_manual_roles(&pool, &user_id).await?;
    let directory_roles = repo::list_directory_roles(&pool, &user_id).await?;
    Ok(Json(UserRolesResponse {
        roles,
        manual_roles,
        directory_roles,
    })
    .into_response())
}

pub(crate) async fn set_user_roles_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
    Json(body): Json<SetUserRolesRequest>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id);
    let before = require_effective_roles(&pool, &user_id).await?;
    let directory_roles = repo::list_directory_roles(&pool, &user_id).await?;
    let platform_admin_count = repo::count_platform_admins(&pool).await?;

    let after = normalized(&body.roles);
    authorize_role_change(
        &user_ctx.roles,
        &before,
        &after,
        &directory_roles,
        platform_admin_count,
    )
    // Why: every refusal is a rule about who may hold what, so the whole set
    // is a 403 carrying the rule's own words. lint-ok: error-adapt
    .map_err(|refusal| AdminError::Forbidden(refusal.to_string()))?;

    // Why: the manual half is the difference. Storing the whole requested set
    // would turn a directory-held role into a manual grant, and it would then
    // survive the member leaving the AD group.
    let manual: Vec<String> = after
        .iter()
        .filter(|role| !directory_roles.contains(role))
        .cloned()
        .collect();
    repo::set_manual_roles(&pool, &user_id, &manual, &user_ctx.user_id).await?;
    let roles = repo::recompute_roles(&pool, &user_id, None).await?;

    // Why: losing a write-tier role has to take the live credentials with it.
    // A session minted while they were an admin carries that claim until it
    // expires, so demotion without revocation is a demotion that does not
    // take effect until tomorrow.
    if has_any(&before, ROLES_MANAGE) && !has_any(&roles, ROLES_MANAGE) {
        revocation::revoke_user_access(&pool, &user_id).await?;
    }

    let manual_roles = repo::list_manual_roles(&pool, &user_id).await?;
    let directory_roles = repo::list_directory_roles(&pool, &user_id).await?;
    Ok(Json(UserRolesResponse {
        roles,
        manual_roles,
        directory_roles,
    })
    .into_response())
}

// Why: duplicates and surrounding space come from a form, not from a rule, so
// they are cleaned here rather than refused. Role names remain free text;
// permission changes are checked by `authorize_role_change`.
fn normalized(roles: &[String]) -> Vec<String> {
    let mut out: Vec<String> = roles
        .iter()
        .map(|r| r.trim().to_owned())
        .filter(|r| !r.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

async fn require_effective_roles(pool: &PgPool, user_id: &UserId) -> AdminResult<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT roles AS "roles!: Vec<String>" FROM users WHERE id = $1"#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AdminError::NotFound(format!("User {user_id} not found")))
}
