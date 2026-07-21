use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::secrets::secret_audit::{self, AuditLogRow};
use crate::repositories::secrets::{secret_crypto, secret_keys, secret_resolve};

pub(crate) async fn create_resolution_token(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> AdminResult<String> {
    let token = secret_resolve::create_resolution_token(pool, user_id, plugin_id).await?;
    Ok(token)
}

pub(crate) async fn resolve_secrets(
    pool: &PgPool,
    plugin_id: &str,
    raw_token: &str,
) -> AdminResult<HashMap<String, String>> {
    let (user_id_str, token_plugin_id) =
        secret_resolve::validate_and_consume_token(pool, raw_token)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, "Token validation failed");
                AdminError::Unauthorized("Invalid or expired token".to_owned())
            })?;

    if token_plugin_id != plugin_id {
        return Err(AdminError::Forbidden("Token plugin mismatch".to_owned()));
    }

    let master_key = secret_crypto::load_master_key()?;
    let user_id = UserId::new(&user_id_str);
    let secrets =
        secret_resolve::resolve_secrets_for_plugin(pool, &user_id, plugin_id, &master_key).await?;
    Ok(secrets)
}

pub(crate) async fn list_audit_log(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> AdminResult<Vec<AuditLogRow>> {
    let rows = secret_audit::list_audit_log(pool, user_id, plugin_id).await?;
    Ok(rows)
}

pub(crate) async fn rotate_user_keys(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> AdminResult<()> {
    let master_key = secret_crypto::load_master_key()?;
    secret_keys::rotate_user_dek(pool, user_id, &master_key).await?;

    if let Err(e) = secret_audit::insert_audit_entry(pool, user_id, plugin_id, "rotated").await {
        tracing::warn!(error = %e, "Failed to insert secret audit log");
    }
    Ok(())
}
