use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PluginEnvVar {
    pub id: String,
    pub plugin_id: String,
    pub var_name: String,
    pub var_value: String,
    pub is_secret: bool,
}

pub async fn list_plugin_env_vars(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<Vec<PluginEnvVar>, MarketplaceError> {
    let rows = sqlx::query_as!(
        PluginEnvVar,
        "SELECT id, plugin_id, var_name, var_value, is_secret \
         FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2 ORDER BY var_name",
        user_id.as_str(),
        plugin_id,
    )
    .fetch_all(pool)
    .await?;

    let rows = if rows.is_empty() && user_id.as_str() != "admin" {
        sqlx::query_as!(
            PluginEnvVar,
            "SELECT id, plugin_id, var_name, var_value, is_secret \
             FROM plugin_env_vars WHERE user_id = 'admin' AND plugin_id = $1 ORDER BY var_name",
            plugin_id,
        )
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, plugin_id = %plugin_id, "Failed to fetch admin fallback env vars");
            Vec::new()
        })
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
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
    var_name: &str,
    var_value: &str,
    is_secret: bool,
) -> Result<(), MarketplaceError> {
    let id = uuid::Uuid::new_v4().to_string();

    if is_secret {
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
            "INSERT INTO plugin_env_vars \
             (id, user_id, plugin_id, var_name, var_value, is_secret, encrypted_value, value_nonce, key_version) \
             VALUES ($1, $2, $3, $4, '', $5, $6, $7, 1) \
             ON CONFLICT (user_id, plugin_id, var_name) DO UPDATE \
             SET var_value = '', is_secret = EXCLUDED.is_secret, \
             encrypted_value = EXCLUDED.encrypted_value, value_nonce = EXCLUDED.value_nonce, \
             key_version = EXCLUDED.key_version, updated_at = NOW()",
            id,
            user_id.as_str(),
            plugin_id,
            var_name,
            is_secret,
            encrypted.as_slice(),
            nonce.as_slice(),
        )
        .execute(pool)
        .await?;

        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO plugin_env_vars (id, user_id, plugin_id, var_name, var_value, is_secret) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id, plugin_id, var_name) DO UPDATE \
         SET var_value = EXCLUDED.var_value, is_secret = EXCLUDED.is_secret, updated_at = NOW()",
        id,
        user_id.as_str(),
        plugin_id,
        var_name,
        var_value,
        is_secret,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_raw_env_vars_for_export(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
) -> Result<std::collections::HashMap<String, String>, MarketplaceError> {
    let rows = sqlx::query!(
        "SELECT var_name, var_value, is_secret FROM plugin_env_vars \
         WHERE user_id = $1 AND plugin_id = $2 ORDER BY var_name",
        user_id.as_str(),
        plugin_id,
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() && user_id.as_str() != "admin" {
        let fallback = sqlx::query!(
            "SELECT var_name, var_value, is_secret FROM plugin_env_vars \
             WHERE user_id = 'admin' AND plugin_id = $1 ORDER BY var_name",
            plugin_id,
        )
        .fetch_all(pool)
        .await?;
        return Ok(fallback
            .into_iter()
            .map(|r| {
                (
                    r.var_name,
                    if r.is_secret {
                        String::new()
                    } else {
                        r.var_value
                    },
                )
            })
            .collect());
    }

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.var_name,
                if r.is_secret {
                    String::new()
                } else {
                    r.var_value
                },
            )
        })
        .collect())
}

pub async fn list_all_user_env_vars(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<PluginEnvVar>, MarketplaceError> {
    let rows = sqlx::query_as!(
        PluginEnvVar,
        "SELECT id, plugin_id, var_name, var_value, is_secret \
         FROM plugin_env_vars WHERE user_id = $1 ORDER BY plugin_id, var_name",
        user_id.as_str(),
    )
    .fetch_all(pool)
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

pub async fn delete_plugin_env_var(
    pool: &PgPool,
    user_id: &UserId,
    plugin_id: &str,
    var_name: &str,
) -> Result<bool, MarketplaceError> {
    let result = sqlx::query!(
        "DELETE FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2 AND var_name = $3",
        user_id.as_str(),
        plugin_id,
        var_name,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
