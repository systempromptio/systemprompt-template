use std::collections::HashMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};
use sqlx::PgPool;

use super::secret_crypto;
use super::secret_keys;

pub async fn create_resolution_token(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<String, anyhow::Error> {
    let raw_token = uuid::Uuid::new_v4().to_string();
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));
    let id = uuid::Uuid::new_v4().to_string();

    sqlx::query(
        "INSERT INTO secret_resolution_tokens (id, user_id, plugin_id, token_hash, expires_at) \
         VALUES ($1, $2, $3, $4, NOW() + INTERVAL '5 minutes')",
    )
    .bind(&id)
    .bind(user_id)
    .bind(plugin_id)
    .bind(&token_hash)
    .execute(pool.as_ref())
    .await?;

    tracing::debug!(user_id = %user_id, plugin_id = %plugin_id, "Created resolution token");
    Ok(raw_token)
}

pub async fn validate_and_consume_token(
    pool: &Arc<PgPool>,
    raw_token: &str,
) -> Result<(String, String), anyhow::Error> {
    let token_hash = hex::encode(Sha256::digest(raw_token.as_bytes()));

    let row = sqlx::query_as::<_, (String, String)>(
        "UPDATE secret_resolution_tokens SET used_at = NOW() \
         WHERE token_hash = $1 AND expires_at > NOW() AND used_at IS NULL \
         RETURNING user_id, plugin_id",
    )
    .bind(&token_hash)
    .fetch_optional(pool.as_ref())
    .await?;

    match row {
        Some((user_id, plugin_id)) => {
            tracing::debug!(user_id = %user_id, plugin_id = %plugin_id, "Consumed resolution token");
            Ok((user_id, plugin_id))
        }
        None => Err(anyhow::anyhow!("Invalid or expired token")),
    }
}

pub async fn resolve_secrets_for_plugin(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
    master_key: &[u8; 32],
) -> Result<HashMap<String, String>, anyhow::Error> {
    let dek = secret_keys::get_or_create_user_dek(pool, user_id, master_key).await?;

    let rows = sqlx::query_as::<_, (String, Vec<u8>, Vec<u8>, i32)>(
        "SELECT var_name, encrypted_value, value_nonce, key_version \
         FROM plugin_env_vars \
         WHERE user_id = $1 AND plugin_id = $2 AND is_secret = true \
         AND encrypted_value IS NOT NULL AND key_version > 0",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_all(pool.as_ref())
    .await?;

    let mut secrets = HashMap::new();
    for (var_name, encrypted_value, value_nonce, _key_version) in &rows {
        let nonce: [u8; 12] = value_nonce
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length for var {var_name}"))?;
        let plaintext = secret_crypto::decrypt(&dek, &nonce, encrypted_value)?;
        let value = String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Decrypted value is not valid UTF-8: {e}"))?;
        secrets.insert(var_name.clone(), value);
    }

    let audit_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', 'accessed', $2)",
    )
    .bind(&audit_id)
    .bind(user_id)
    .bind(plugin_id)
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
