//! Resolves the caller's access scope from verified sources only: the token's
//! permissions and the user's stored roles.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::Permission;
use systemprompt_security::policy::types::AccessScope;

pub(super) async fn scope_from_user_roles(pool: &PgPool, user_id: &UserId) -> AccessScope {
    match crate::repositories::users::queries::find_user_roles_department(pool, user_id).await {
        Ok(Some((roles, _dept))) => {
            if roles.iter().any(|r| r == "admin") {
                AccessScope::Admin
            } else if roles.iter().any(|r| r == "user") {
                AccessScope::User
            } else {
                AccessScope::Unknown
            }
        },
        Ok(None) => AccessScope::Unknown,
        Err(e) => {
            tracing::warn!(
                error = %e,
                %user_id,
                "governance: user role lookup failed; no DB-derived scope"
            );
            AccessScope::Unknown
        },
    }
}

pub(super) fn scope_from_permissions(perms: &[Permission]) -> AccessScope {
    if perms.contains(&Permission::Admin) {
        AccessScope::Admin
    } else if perms.contains(&Permission::User) {
        AccessScope::User
    } else {
        AccessScope::Unknown
    }
}

pub(super) const fn higher_privilege(a: AccessScope, b: AccessScope) -> AccessScope {
    match (a, b) {
        (AccessScope::Admin, _) | (_, AccessScope::Admin) => AccessScope::Admin,
        (AccessScope::User, _) | (_, AccessScope::User) => AccessScope::User,
        _ => AccessScope::Unknown,
    }
}
