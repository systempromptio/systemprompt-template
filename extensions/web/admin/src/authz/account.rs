//! Resolves access scope from the account's current status and stored roles.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::policy::types::AccessScope;

pub async fn account_scope(pool: &PgPool, user_id: &UserId) -> Result<AccessScope, sqlx::Error> {
    let identity = crate::repositories::users::queries::find_identity_envelope(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if identity.status != "active" {
        return Ok(AccessScope::Unknown);
    }
    Ok(if crate::types::roles_grant_manage(&identity.roles) {
        AccessScope::Admin
    } else {
        AccessScope::User
    })
}
