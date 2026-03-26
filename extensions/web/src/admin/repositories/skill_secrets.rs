use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;

use crate::admin::types::SkillSecret;

pub async fn list_skill_secrets(
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
) -> Result<Vec<SkillSecret>, anyhow::Error> {
    let rows = sqlx::query_as::<_, SkillSecret>(
        "SELECT id, skill_id, var_name, var_value, is_secret \
         FROM skill_secrets WHERE user_id = $1 AND skill_id = $2 ORDER BY var_name",
    )
    .bind(user_id)
    .bind(skill_id)
    .fetch_all(pool.as_ref())
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
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
    var_name: &str,
    var_value: &str,
) -> Result<(), anyhow::Error> {
    let id = uuid::Uuid::new_v4().to_string();

    let master_key = super::secret_crypto::load_master_key()
        .map_err(|e| anyhow::anyhow!("Cannot store secret — encryption not configured: {e}"))?;
    let dek = super::secret_keys::get_or_create_user_dek(pool, user_id, &master_key)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let nonce = super::secret_crypto::generate_nonce();
    let encrypted = super::secret_crypto::encrypt(&dek, &nonce, var_value.as_bytes())
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    sqlx::query(
        "INSERT INTO skill_secrets \
         (id, user_id, skill_id, var_name, var_value, is_secret, encrypted_value, value_nonce, key_version) \
         VALUES ($1, $2, $3, $4, '', true, $5, $6, 1) \
         ON CONFLICT (user_id, skill_id, var_name) DO UPDATE \
         SET var_value = '', is_secret = true, \
         encrypted_value = EXCLUDED.encrypted_value, value_nonce = EXCLUDED.value_nonce, \
         key_version = EXCLUDED.key_version, updated_at = NOW()",
    )
    .bind(&id)
    .bind(user_id)
    .bind(skill_id)
    .bind(var_name)
    .bind(&encrypted)
    .bind(nonce.as_slice())
    .execute(pool.as_ref())
    .await?;

    Ok(())
}

pub async fn delete_skill_secret(
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
    var_name: &str,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "DELETE FROM skill_secrets WHERE user_id = $1 AND skill_id = $2 AND var_name = $3",
    )
    .bind(user_id)
    .bind(skill_id)
    .bind(var_name)
    .execute(pool.as_ref())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_all_user_skill_secrets(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<SkillSecret>, anyhow::Error> {
    let rows = sqlx::query_as::<_, SkillSecret>(
        "SELECT id, skill_id, var_name, var_value, is_secret \
         FROM skill_secrets WHERE user_id = $1 ORDER BY skill_id, var_name",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
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
    pool: &Arc<PgPool>,
    user_id: &str,
    skill_id: &str,
    master_key: &[u8; 32],
) -> Result<HashMap<String, String>, anyhow::Error> {
    let dek = super::secret_keys::get_or_create_user_dek(pool, user_id, master_key).await?;

    let rows = sqlx::query_as::<_, (String, Vec<u8>, Vec<u8>, i32)>(
        "SELECT var_name, encrypted_value, value_nonce, key_version \
         FROM skill_secrets \
         WHERE user_id = $1 AND skill_id = $2 AND is_secret = true \
         AND encrypted_value IS NOT NULL AND key_version > 0",
    )
    .bind(user_id)
    .bind(skill_id)
    .fetch_all(pool.as_ref())
    .await?;

    let mut secrets = HashMap::new();
    for (var_name, encrypted_value, value_nonce, _key_version) in &rows {
        let nonce: [u8; 12] = value_nonce
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid nonce length for var {var_name}"))?;
        let plaintext = super::secret_crypto::decrypt(&dek, &nonce, encrypted_value)?;
        let value = String::from_utf8(plaintext)
            .map_err(|e| anyhow::anyhow!("Decrypted value is not valid UTF-8: {e}"))?;
        secrets.insert(var_name.clone(), value);
    }

    let audit_id = uuid::Uuid::new_v4().to_string();
    if let Err(e) = sqlx::query(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, $3, '*', 'accessed', $2)",
    )
    .bind(&audit_id)
    .bind(user_id)
    .bind(format!("skill:{skill_id}"))
    .execute(pool.as_ref())
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
