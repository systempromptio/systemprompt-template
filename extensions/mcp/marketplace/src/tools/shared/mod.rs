mod plugin;
mod slugs;

use std::sync::Arc;

use sqlx::PgPool;

pub use plugin::{add_to_plugin, auto_add_to_default_plugin};
pub use slugs::{
    resolve_agent_slugs, resolve_agent_uuids_to_slugs, resolve_mcp_server_slugs,
    resolve_mcp_server_uuids_to_slugs, resolve_skill_slugs, resolve_skill_uuids_to_slugs,
};

use systemprompt::database::DbPool;
use systemprompt::identifiers::UserId;
use systemprompt::mcp::McpError;
use systemprompt_web_extension::admin::repositories;

pub fn require_write_pool(db_pool: &DbPool) -> Result<Arc<PgPool>, McpError> {
    db_pool
        .write_pool()
        .ok_or_else(|| McpError::internal_error("Database pool not available".to_string(), None))
}

pub fn require_pool(db_pool: &DbPool) -> Result<Arc<PgPool>, McpError> {
    db_pool
        .pool()
        .ok_or_else(|| McpError::internal_error("Database pool not available".to_string(), None))
}

#[must_use]
pub fn generate_slug(name: &str) -> String {
    let slug: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    if slug.is_empty() {
        uuid::Uuid::new_v4().to_string()
    } else {
        slug
    }
}

pub async fn resolve_association_slugs(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    plugin_id: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let Ok(Some(assoc)) =
        repositories::get_plugin_with_associations(pool, user_id, plugin_id).await
    else {
        return (vec![], vec![], vec![]);
    };
    let skill_strs: Vec<String> = assoc
        .skill_ids
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let agent_strs: Vec<String> = assoc
        .agent_ids
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let mcp_strs: Vec<String> = assoc
        .mcp_server_ids
        .iter()
        .map(std::string::ToString::to_string)
        .collect();
    let (skills, agents, mcps) = tokio::join!(
        resolve_skill_uuids_to_slugs(pool, &skill_strs),
        resolve_agent_uuids_to_slugs(pool, &agent_strs),
        resolve_mcp_server_uuids_to_slugs(pool, &mcp_strs),
    );
    (skills, agents, mcps)
}

pub async fn invalidate_marketplace_cache(pool: &Arc<PgPool>, user_id: &UserId) {
    if let Err(e) = repositories::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty after MCP mutation");
    }
    if let Err(e) =
        systemprompt_web_extension::admin::repositories::marketplace_sync::invalidate_git_cache(
            user_id,
        )
    {
        tracing::warn!(error = %e, "Failed to invalidate git cache after MCP mutation");
    }
}
