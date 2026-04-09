use std::collections::HashMap;

use sqlx::PgPool;

use systemprompt::identifiers::{SkillId, UserId};

use super::super::types::SkillSecret;
use systemprompt_web_shared::error::MarketplaceError;

pub async fn list_skill_secrets(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
) -> Result<Vec<SkillSecret>, MarketplaceError> {
    let rows = sqlx::query_as!(
        SkillSecret,
        "SELECT id, skill_id, var_name, var_value, is_secret \
         FROM skill_secrets WHERE user_id = $1 AND skill_id = $2 ORDER BY var_name",
        user_id.as_str(),
        skill_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let masked: Vec<SkillSecret> = rows
        .into_iter()
        .map(|mut r| {
            if r.is_secret && !r.var_value.is_empty() {
                r.var_value = "••••••••".to_string();
            }
            r
        })
        .collect();

    Ok(masked)
}

pub async fn upsert_skill_secret(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
    var_name: &str,
    var_value: &str,
) -> Result<(), MarketplaceError> {
    let id = uuid::Uuid::new_v4().to_string();

    let master_key = super::secret_crypto::load_master_key().map_err(|e| {
        MarketplaceError::Internal(format!(
            "Cannot store secret — encryption not configured: {e}"
        ))
    })?;
    let dek = super::secret_keys::get_or_create_user_dek(pool, user_id, &master_key)
        .await
        .map_err(|e| MarketplaceError::Internal(format!("{e}")))?;
    let nonce = super::secret_crypto::generate_nonce();
    let encrypted = super::secret_crypto::encrypt(&dek, &nonce, var_value.as_bytes())
        .map_err(|e| MarketplaceError::Internal(format!("{e}")))?;

    sqlx::query!(
        "INSERT INTO skill_secrets \
         (id, user_id, skill_id, var_name, var_value, is_secret, encrypted_value, value_nonce, key_version) \
         VALUES ($1, $2, $3, $4, '', true, $5, $6, 1) \
         ON CONFLICT (user_id, skill_id, var_name) DO UPDATE \
         SET var_value = '', is_secret = true, \
         encrypted_value = EXCLUDED.encrypted_value, value_nonce = EXCLUDED.value_nonce, \
         key_version = EXCLUDED.key_version, updated_at = NOW()",
        id,
        user_id.as_str(),
        skill_id.as_str(),
        var_name,
        encrypted.as_slice(),
        nonce.as_slice(),
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_skill_secret(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
    var_name: &str,
) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "DELETE FROM skill_secrets WHERE user_id = $1 AND skill_id = $2 AND var_name = $3",
        user_id.as_str(),
        skill_id.as_str(),
        var_name,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_all_user_skill_secrets(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SkillSecret>, MarketplaceError> {
    let rows = sqlx::query_as!(
        SkillSecret,
        "SELECT id, skill_id, var_name, var_value, is_secret \
         FROM skill_secrets WHERE user_id = $1 ORDER BY skill_id, var_name",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let masked: Vec<SkillSecret> = rows
        .into_iter()
        .map(|mut r| {
            if r.is_secret && !r.var_value.is_empty() {
                r.var_value = "••••••••".to_string();
            }
            r
        })
        .collect();

    Ok(masked)
}

pub async fn resolve_secrets_for_skill(
    pool: &PgPool,
    user_id: &UserId,
    skill_id: &SkillId,
    master_key: &[u8; 32],
) -> Result<HashMap<String, String>, MarketplaceError> {
    let dek = super::secret_keys::get_or_create_user_dek(pool, user_id, master_key).await?;

    let rows = sqlx::query!(
        "SELECT var_name, encrypted_value, value_nonce, key_version \
         FROM skill_secrets \
         WHERE user_id = $1 AND skill_id = $2 AND is_secret = true \
         AND encrypted_value IS NOT NULL AND key_version > 0",
        user_id.as_str(),
        skill_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let mut secrets = HashMap::new();
    for row in &rows {
        let encrypted_value = row.encrypted_value.as_deref().unwrap_or(&[]);
        let value_nonce = row.value_nonce.as_deref().unwrap_or(&[]);
        let nonce: [u8; 12] = value_nonce.try_into().map_err(|_| {
            MarketplaceError::Internal(format!("Invalid nonce length for var {}", row.var_name))
        })?;
        let plaintext = super::secret_crypto::decrypt(&dek, &nonce, encrypted_value)?;
        let value = String::from_utf8(plaintext).map_err(|e| {
            MarketplaceError::Internal(format!("Decrypted value is not valid UTF-8: {e}"))
        })?;
        secrets.insert(row.var_name.clone(), value);
    }

    let audit_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = sqlx::query!(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', 'accessed', $2)",
        audit_id,
        user_id.as_str(),
        format!("skill:{}", skill_id.as_str()),
    )
    .execute(pool)
    .await
    {
        tracing::warn!(error = %e, "Failed to insert secret audit log");
    }

    tracing::info!(
        user_id = %user_id,
        skill_id = %skill_id,
        count = secrets.len(),
        "Resolved secrets for skill"
    );
    Ok(secrets)
}
