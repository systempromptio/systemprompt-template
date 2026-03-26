use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{UserPluginWithAssociations};
use super::user_plugins::get_user_plugin;

pub async fn get_plugin_with_associations(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<Option<UserPluginWithAssociations>, sqlx::Error> {
    let plugin = get_user_plugin(pool, user_id, plugin_id).await?;
    let Some(plugin) = plugin else {
        return Ok(None);
    };

    let skill_ids: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT user_skill_id FROM user_plugin_skills WHERE user_plugin_id = $1 ORDER BY sort_order",
    )
    .bind(&plugin.id)
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|(id,)| id)
    .collect();

    let agent_ids: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT user_agent_id FROM user_plugin_agents WHERE user_plugin_id = $1 ORDER BY sort_order",
    )
    .bind(&plugin.id)
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|(id,)| id)
    .collect();

    let mcp_server_ids: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT user_mcp_server_id FROM user_plugin_mcp_servers WHERE user_plugin_id = $1 ORDER BY sort_order",
    )
    .bind(&plugin.id)
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|(id,)| id)
    .collect();

    let hook_ids: Vec<String> = sqlx::query_as::<_, (String,)>(
        "SELECT user_hook_id FROM user_plugin_hooks WHERE user_plugin_id = $1 ORDER BY sort_order",
    )
    .bind(&plugin.id)
    .fetch_all(pool.as_ref())
    .await?
    .into_iter()
    .map(|(id,)| id)
    .collect();

    Ok(Some(UserPluginWithAssociations {
        plugin,
        skill_ids,
        agent_ids,
        mcp_server_ids,
        hook_ids,
    }))
}
