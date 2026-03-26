use std::sync::Arc;

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PluginEnvVar {
    pub id: String,
    pub plugin_id: String,
    pub var_name: String,
    pub var_value: String,
    pub is_secret: bool,
}

pub async fn list_plugin_env_vars(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<Vec<PluginEnvVar>, anyhow::Error> {
    let rows = sqlx::query_as::<_, PluginEnvVar>(
        "SELECT id, plugin_id, var_name, var_value, is_secret \
         FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2 ORDER BY var_name",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_all(pool.as_ref())
    .await?;

    let rows = if rows.is_empty() && user_id != "admin" {
        sqlx::query_as::<_, PluginEnvVar>(
            "SELECT id, plugin_id, var_name, var_value, is_secret \
             FROM plugin_env_vars WHERE user_id = 'admin' AND plugin_id = $1 ORDER BY var_name",
        )
        .bind(plugin_id)
        .fetch_all(pool.as_ref())
        .await
        .unwrap_or_else(|_| Vec::new())
    } else {
        rows
    };

    let masked: Vec<PluginEnvVar> = rows
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

pub async fn upsert_plugin_env_var(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
    var_name: &str,
    var_value: &str,
    is_secret: bool,
) -> Result<(), anyhow::Error> {
    let id = uuid::Uuid::new_v4().to_string();

    if is_secret {
        if let Ok(master_key) = super::secret_crypto::load_master_key() {
            let dek = super::secret_keys::get_or_create_user_dek(pool, user_id, &master_key)
                .await
                .map_err(|e| anyhow::anyhow!("{e}"))?;
            let nonce = super::secret_crypto::generate_nonce();
            let encrypted = super::secret_crypto::encrypt(&dek, &nonce, var_value.as_bytes())
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            sqlx::query(
                "INSERT INTO plugin_env_vars \
                 (id, user_id, plugin_id, var_name, var_value, is_secret, encrypted_value, value_nonce, key_version) \
                 VALUES ($1, $2, $3, $4, '', $5, $6, $7, 1) \
                 ON CONFLICT (user_id, plugin_id, var_name) DO UPDATE \
                 SET var_value = '', is_secret = EXCLUDED.is_secret, \
                 encrypted_value = EXCLUDED.encrypted_value, value_nonce = EXCLUDED.value_nonce, \
                 key_version = EXCLUDED.key_version, updated_at = NOW()",
            )
            .bind(&id)
            .bind(user_id)
            .bind(plugin_id)
            .bind(var_name)
            .bind(is_secret)
            .bind(&encrypted)
            .bind(nonce.as_slice())
            .execute(pool.as_ref())
            .await?;

            return Ok(());
        }
        tracing::warn!("ENCRYPTION_MASTER_KEY not set, storing secret in plaintext");
    }

    sqlx::query(
        "INSERT INTO plugin_env_vars (id, user_id, plugin_id, var_name, var_value, is_secret) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id, plugin_id, var_name) DO UPDATE \
         SET var_value = EXCLUDED.var_value, is_secret = EXCLUDED.is_secret, updated_at = NOW()",
    )
    .bind(&id)
    .bind(user_id)
    .bind(plugin_id)
    .bind(var_name)
    .bind(var_value)
    .bind(is_secret)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

pub async fn delete_plugin_env_var(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
    var_name: &str,
) -> Result<bool, anyhow::Error> {
    let result = sqlx::query(
        "DELETE FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2 AND var_name = $3",
    )
    .bind(user_id)
    .bind(plugin_id)
    .bind(var_name)
    .execute(pool.as_ref())
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn list_all_user_env_vars(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<PluginEnvVar>, anyhow::Error> {
    let rows = sqlx::query_as::<_, PluginEnvVar>(
        "SELECT id, plugin_id, var_name, var_value, is_secret \
         FROM plugin_env_vars WHERE user_id = $1 ORDER BY plugin_id, var_name",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await?;

    let masked: Vec<PluginEnvVar> = rows
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

pub async fn get_raw_env_vars_for_export(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<std::collections::HashMap<String, String>, anyhow::Error> {
    let rows = sqlx::query_as::<_, (String, String, bool)>(
        "SELECT var_name, var_value, is_secret FROM plugin_env_vars \
         WHERE user_id = $1 AND plugin_id = $2 ORDER BY var_name",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_all(pool.as_ref())
    .await?;

    if rows.is_empty() && user_id != "admin" {
        let fallback = sqlx::query_as::<_, (String, String, bool)>(
            "SELECT var_name, var_value, is_secret FROM plugin_env_vars \
             WHERE user_id = 'admin' AND plugin_id = $1 ORDER BY var_name",
        )
        .bind(plugin_id)
        .fetch_all(pool.as_ref())
        .await?;
        return Ok(fallback
            .into_iter()
            .map(|(name, val, is_secret)| {
                (name, if is_secret { String::new() } else { val })
            })
            .collect());
    }

    Ok(rows
        .into_iter()
        .map(|(name, val, is_secret)| {
            (name, if is_secret { String::new() } else { val })
        })
        .collect())
}
