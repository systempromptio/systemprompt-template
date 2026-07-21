//! Per-plugin environment variable records.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, UserId};
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct PluginEnvVar {
    pub id: String,
    pub plugin_id: PluginId,
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
        "SELECT id, plugin_id AS \"plugin_id: PluginId\", var_name, var_value, is_secret \
         FROM plugin_env_vars WHERE user_id = $1 AND plugin_id = $2 ORDER BY var_name",
        user_id.as_str(),
        plugin_id,
    )
    .fetch_all(pool)
    .await?;

    let rows = if rows.is_empty() && user_id.as_str() != "admin" {
        sqlx::query_as!(
            PluginEnvVar,
            "SELECT id, plugin_id AS \"plugin_id: PluginId\", var_name, var_value, is_secret \
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
                "••••••••".clone_into(&mut r.var_value);
            }
            r
        })
        .collect();

    Ok(masked)
}
