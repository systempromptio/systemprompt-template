use std::collections::HashMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use super::secret_crypto;
use super::secret_keys;
use crate::error::MarketplaceError;

pub async fn create_resolution_token(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<String, MarketplaceError> {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query!(
        "INSERT INTO secret_resolution_tokens (id, user_id, plugin_id, token_hash, expires_at) \
         VALUES ($1, $2, $3, $4, NOW() + INTERVAL '5 minutes')",
        id,
        user_id.as_str(),
        plugin_id,
        token_hash,
    )
    .execute(pool.as_ref())
    .await?;

    tracing::debug!(user_id = %user_id, plugin_id = %plugin_id, "Created resolution token");
    Ok(raw_token)
}

pub async fn validate_and_consume_token(
    pool: &Arc<PgPool>,
    raw_token: &str,
) -> Result<(String, String), MarketplaceError> {
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));

    let row = sqlx::query!(
        "UPDATE secret_resolution_tokens SET used_at = NOW() \
         WHERE token_hash = $1 AND expires_at > NOW() AND used_at IS NULL \
         RETURNING user_id, plugin_id",
        token_hash,
    )
    .fetch_optional(pool.as_ref())
    .await?;

    match row {
        Some(r) => {
            tracing::debug!(user_id = %r.user_id, plugin_id = %r.plugin_id, "Consumed resolution token");
            Ok((r.user_id, r.plugin_id))
        }
        None => Err(MarketplaceError::Internal(
            "Invalid or expired token".to_string(),
        )),
    }
}

pub async fn resolve_secrets_for_plugin(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    plugin_id: &str,
    master_key: &[u8; 32],
) -> Result<HashMap<String, String>, MarketplaceError> {
    let dek = secret_keys::get_or_create_user_dek(pool, user_id, master_key).await?;

    let rows = sqlx::query!(
        "SELECT var_name, encrypted_value, value_nonce, key_version \
         FROM plugin_env_vars \
         WHERE user_id = $1 AND plugin_id = $2 AND is_secret = true \
         AND encrypted_value IS NOT NULL AND key_version > 0",
        user_id.as_str(),
        plugin_id,
    )
    .fetch_all(pool.as_ref())
    .await?;

    let mut secrets = HashMap::new();
    for row in &rows {
        let encrypted_value = row.encrypted_value.as_deref().unwrap_or(&[]);
        let value_nonce = row.value_nonce.as_deref().unwrap_or(&[]);
        let nonce: [u8; 12] = value_nonce.try_into().map_err(|_| {
            MarketplaceError::Internal(format!("Invalid nonce length for var {}", row.var_name))
        })?;
        let plaintext = secret_crypto::decrypt(&dek, &nonce, encrypted_value)?;
        let value = String::from_utf8(plaintext).map_err(|e| {
            MarketplaceError::Internal(format!("Decrypted value is not valid UTF-8: {e}"))
        })?;
        secrets.insert(row.var_name.clone(), value);
    }

    let audit_id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', 'accessed', $2)",
        audit_id,
        user_id.as_str(),
        plugin_id,
    )
    .execute(pool.as_ref())
    .await?;

    tracing::info!(
        user_id = %user_id,
        plugin_id = %plugin_id,
        count = secrets.len(),
        "Resolved secrets for plugin"
    );
    Ok(secrets)
}
