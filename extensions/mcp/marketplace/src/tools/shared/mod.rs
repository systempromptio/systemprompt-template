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
) -> Result<(Vec<String>, Vec<String>, Vec<String>), McpError> {
    let Ok(Some(assoc)) =
        repositories::get_plugin_with_associations(pool, user_id, plugin_id).await
    else {
        return Ok((vec![], vec![], vec![]));
    };
    let skill_strs: Vec<String> = assoc.skill_ids.iter().map(ToString::to_string).collect();
    let agent_strs: Vec<String> = assoc.agent_ids.iter().map(ToString::to_string).collect();
    let mcp_strs: Vec<String> = assoc
        .mcp_server_ids
        .iter()
        .map(ToString::to_string)
        .collect();
    let (skills, agents, mcps) = tokio::try_join!(
        resolve_skill_uuids_to_slugs(&**pool, &skill_strs),
        resolve_agent_uuids_to_slugs(&**pool, &agent_strs),
        resolve_mcp_server_uuids_to_slugs(&**pool, &mcp_strs),
    )?;
    Ok((skills, agents, mcps))
}

pub async fn set_plugin_associations(
    conn: &mut sqlx::PgConnection,
    plugin_id: &str,
    user_id: &UserId,
    skill_slugs: Option<&[String]>,
    agent_slugs: Option<&[String]>,
    mcp_server_slugs: Option<&[String]>,
) -> Result<(), McpError> {
    if let Some(slugs) = skill_slugs {
        if !slugs.is_empty() {
            let uuids = resolve_skill_slugs(&mut *conn, user_id.as_ref(), slugs).await?;
            let typed: Vec<_> = uuids
                .into_iter()
                .map(systemprompt::identifiers::SkillId::new)
                .collect();
            repositories::set_plugin_skills(&mut *conn, plugin_id, &typed)
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to set plugin skills: {e}"), None)
                })?;
        }
    }
    if let Some(slugs) = agent_slugs {
        if !slugs.is_empty() {
            let uuids = resolve_agent_slugs(&mut *conn, user_id.as_ref(), slugs).await?;
            let typed: Vec<_> = uuids
                .into_iter()
                .map(systemprompt::identifiers::AgentId::new)
                .collect();
            repositories::set_plugin_agents(&mut *conn, plugin_id, &typed)
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to set plugin agents: {e}"), None)
                })?;
        }
    }
    if let Some(slugs) = mcp_server_slugs {
        if !slugs.is_empty() {
            let uuids = resolve_mcp_server_slugs(&mut *conn, user_id.as_ref(), slugs).await?;
            let typed: Vec<_> = uuids
                .into_iter()
                .map(systemprompt::identifiers::McpServerId::new)
                .collect();
            repositories::set_plugin_mcp_servers(&mut *conn, plugin_id, &typed)
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to set plugin MCP servers: {e}"), None)
                })?;
        }
    }
    Ok(())
}

pub fn build_plugin_response(
    plugin: &systemprompt_web_extension::admin::types::UserPlugin,
    ctx: &systemprompt::models::execution::context::RequestContext,
    action: &str,
    skill_slugs: &[String],
    agent_slugs: &[String],
    mcp_server_slugs: &[String],
) -> Result<(systemprompt::models::artifacts::TextArtifact, String), McpError> {
    use systemprompt::models::artifacts::TextArtifact;

    let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
        "_display": { "type": "card", "entity": "plugin", "action": action },
        "plugin_id": plugin.plugin_id,
        "name": plugin.name,
        "description": plugin.description,
        "version": plugin.version,
        "enabled": plugin.enabled,
        "category": plugin.category,
        "keywords": plugin.keywords,
        "author_name": plugin.author_name,
        "skill_ids": skill_slugs,
        "agent_ids": agent_slugs,
        "mcp_server_ids": mcp_server_slugs,
        "created_at": plugin.created_at.to_rfc3339(),
        "updated_at": plugin.updated_at.to_rfc3339(),
    }))
    .map_err(|e| McpError::internal_error(format!("Failed to serialize plugin: {e}"), None))?;

    let action_past = if action == "created" {
        "Created"
    } else {
        "Updated"
    };
    let summary = format!(
        "{action_past} plugin '{}' ({})",
        plugin.name, plugin.plugin_id
    );
    let content = format!("{summary}\n\n{plugin_json}");
    let artifact =
        TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

    Ok((artifact, content))
}

pub async fn invalidate_marketplace_cache(pool: &Arc<PgPool>, user_id: &UserId) {
    if let Err(e) = repositories::mark_user_dirty(pool, user_id).await {
        tracing::warn!(error = %e, "Failed to mark user dirty after MCP mutation");
    }
    if let Err(e) = repositories::marketplace_sync::invalidate_git_cache(user_id) {
        tracing::warn!(error = %e, "Failed to invalidate git cache after MCP mutation");
    }
}
