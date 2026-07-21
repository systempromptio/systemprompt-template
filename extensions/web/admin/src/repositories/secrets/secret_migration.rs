//! Re-encryption of legacy plaintext secrets, driven by the migration job.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug)]
pub struct UnencryptedSecret {
    pub id: String,
    pub user_id: UserId,
    pub var_name: String,
    pub var_value: String,
}

pub async fn list_unencrypted_secrets(
    pool: &PgPool,
) -> Result<Vec<UnencryptedSecret>, sqlx::Error> {
    let rows = sqlx::query!(
        "SELECT id, user_id, var_name, var_value FROM plugin_env_vars \
         WHERE is_secret = true AND (encrypted_value IS NULL OR key_version = 0) \
         AND var_value != '' LIMIT 100",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UnencryptedSecret {
            id: r.id,
            user_id: r.user_id.into(),
            var_name: r.var_name,
            var_value: r.var_value,
        })
        .collect())
}

pub async fn get_key_version(pool: &PgPool, user_id: &UserId) -> i32 {
    sqlx::query_scalar!(
        "SELECT key_version FROM user_encryption_keys WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_one(pool)
    .await
    .unwrap_or(1)
}

pub async fn update_encrypted_value(
    pool: &PgPool,
    id: &str,
    encrypted: &[u8],
    nonce: &[u8],
    key_version: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, \
         key_version = $3, var_value = '', updated_at = NOW() WHERE id = $4",
        encrypted,
        nonce,
        key_version,
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_migration_audit(
    pool: &PgPool,
    user_id: &UserId,
    var_name: &str,
    actor: &UserId,
) -> Result<(), sqlx::Error> {
    let audit_id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        "INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id) \
         VALUES ($1, $2, '', $3, 'updated', $4)",
        audit_id,
        user_id.as_str(),
        var_name,
        actor.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}
