//! Resolves access scope and roles from the account's current status and
//! stored roles.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::policy::types::AccessScope;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountStanding {
    pub scope: AccessScope,
    pub roles: Vec<String>,
}

pub async fn account_standing(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<AccountStanding, sqlx::Error> {
    let identity = crate::repositories::users::queries::find_identity_envelope(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if identity.status != "active" {
        return Ok(AccountStanding {
            scope: AccessScope::Unknown,
            roles: Vec::new(),
        });
    }
    let scope = if crate::types::roles_grant_manage(&identity.roles) {
        AccessScope::Admin
    } else {
        AccessScope::User
    };
    Ok(AccountStanding {
        scope,
        roles: identity.roles,
    })
}

pub async fn account_scope(pool: &PgPool, user_id: &UserId) -> Result<AccessScope, sqlx::Error> {
    account_standing(pool, user_id)
        .await
        .map(|standing| standing.scope)
}
