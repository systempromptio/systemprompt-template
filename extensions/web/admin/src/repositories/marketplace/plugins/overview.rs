//! Role-scoped marketplace views: counts and overviews filtered by the
//! caller's roles, plus the (currently empty) onboarding config map.


use systemprompt::identifiers::McpServerId;

use crate::repositories::marketplace::plugin_loader;
use crate::repositories::marketplace::plugin_resolvers::{
    resolve_plugin_agents, resolve_plugin_skills,
};
use crate::types::PluginOverview;
use std::path::Path;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy)]
pub struct MarketplaceCounts {
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
}

pub fn list_plugins_for_roles(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<PluginOverview>, MarketplaceError> {
    list_plugins_for_roles_full(services_path, roles)
}

pub fn list_plugins_for_roles_full(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<PluginOverview>, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let is_admin = crate::types::roles_grant_manage(roles);
    let mut overviews = Vec::new();
    for (_id, plugin) in plugin_loader::load_all_plugins()? {
        if !plugin.base.enabled && !is_admin {
            continue;
        }
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        let skill_infos = resolve_plugin_skills(&plugin.base, &skills_path, &agents_path);
        let agent_infos = resolve_plugin_agents(&plugin.base, &agents_path);
        overviews.push(PluginOverview {
            id: plugin.base.id.to_string(),
            name: plugin.base.name,
            description: plugin.base.description,
            enabled: plugin.base.enabled,
            skills: skill_infos,
            agents: agent_infos,
            mcp_servers: plugin
                .base
                .mcp_servers
                .include
                .into_iter()
                .filter_map(|s| McpServerId::try_new(s).ok())
                .collect(),
            hooks: vec![],
            depends: plugin.depends,
        });
    }
    Ok(overviews)
}

pub async fn list_plugins_for_user(
    pool: &sqlx::PgPool,
    services_path: &Path,
    user: &systemprompt::identifiers::UserId,
) -> Result<Vec<PluginOverview>, crate::error::AdminError> {
    use systemprompt_security::authz::EntityKind;
    let mut plugins = list_plugins_for_roles_full(services_path, &["admin".to_owned()])
        .map_err(crate::error::AdminError::internal)?;
    plugins.retain(|p| p.enabled);
    let access = crate::authz::catalog::CatalogAccess::load(pool, user).await?;
    let ids = plugins.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
    let permitted = access.allowed(EntityKind::Plugin, &ids).await?;
    plugins.retain(|p| permitted.contains(&p.id));
    let skills = plugins
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.id.to_string()))
        .collect::<Vec<_>>();
    let agents = plugins
        .iter()
        .flat_map(|p| p.agents.iter().map(|a| a.id.to_string()))
        .collect::<Vec<_>>();
    let servers = plugins
        .iter()
        .flat_map(|p| p.mcp_servers.iter().map(ToString::to_string))
        .collect::<Vec<_>>();
    let skills = access.allowed(EntityKind::Skill, &skills).await?;
    let agents = access.allowed(EntityKind::Agent, &agents).await?;
    let servers = access.allowed(EntityKind::McpServer, &servers).await?;
    for plugin in &mut plugins {
        plugin
            .skills
            .retain(|s| s.enabled && skills.contains(s.id.as_str()));
        plugin
            .agents
            .retain(|a| a.enabled && agents.contains(a.id.as_str()));
        plugin.mcp_servers.retain(|m| servers.contains(m.as_str()));
    }
    Ok(plugins)
}

pub fn count_visible_items(plugins: &[PluginOverview]) -> MarketplaceCounts {
    use std::collections::HashSet;
    MarketplaceCounts {
        total_plugins: plugins.len(),
        total_skills: plugins
            .iter()
            .flat_map(|p| p.skills.iter().map(|s| &s.id))
            .collect::<HashSet<_>>()
            .len(),
        agents_count: plugins
            .iter()
            .flat_map(|p| p.agents.iter().map(|a| &a.id))
            .collect::<HashSet<_>>()
            .len(),
        mcp_count: plugins
            .iter()
            .flat_map(|p| &p.mcp_servers)
            .collect::<HashSet<_>>()
            .len(),
    }
}
