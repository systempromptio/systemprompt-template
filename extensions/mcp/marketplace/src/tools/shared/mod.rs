mod plugin;
mod slugs;

use std::sync::Arc;

use sqlx::PgPool;

pub use plugin::{add_to_plugin, auto_add_to_default_plugin};
pub use slugs::{
    resolve_agent_slugs, resolve_agent_uuids_to_slugs, resolve_mcp_server_slugs,
    resolve_mcp_server_uuids_to_slugs, resolve_skill_slugs, resolve_skill_uuids_to_slugs,
};

use systemprompt_web_extension::admin::repositories;

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

pub async fn invalidate_marketplace_cache(pool: &Arc<PgPool>, user_id: &str) {
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
