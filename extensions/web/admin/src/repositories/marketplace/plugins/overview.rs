//! Role-scoped marketplace views: counts and overviews filtered by the
//! caller's roles, plus the (currently empty) onboarding config map.


use systemprompt::identifiers::McpServerId;

use crate::repositories::marketplace::plugin_loader;
use crate::repositories::marketplace::plugin_resolvers::{
    resolve_all_plugin_skill_ids, resolve_plugin_agents, resolve_plugin_skills,
};
use crate::types::{PluginOverview, ROLE_ADMIN};
use std::path::Path;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone, Copy)]
pub struct MarketplaceCounts {
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
}

pub fn count_marketplace_items(
    services_path: &Path,
    roles: &[String],
) -> Result<MarketplaceCounts, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut counts = MarketplaceCounts {
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
    };

    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
    for (_id, plugin) in plugin_loader::load_all_plugins()? {
        if !plugin.base.enabled {
            continue;
        }
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        counts.total_plugins += 1;
        counts.total_skills +=
            resolve_all_plugin_skill_ids(&plugin.base, &skills_path, &agents_path).len();
        counts.agents_count += plugin.base.agents.include.len();
        counts.mcp_count += plugin.base.mcp_servers.include.len();
    }

    Ok(counts)
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
    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
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
