//! Catalog data loading.
//!
//! `load_catalog` reads the plugins, skills, MCP servers, agents, and hooks
//! from `services/` once and derives the reverse indexes that let a skill or
//! MCP server list the plugins including it. Shaping that snapshot into the
//! `Serialize` page structs lives in [`super::view_models`]; handlers own
//! request flow.

use std::collections::HashMap;

use crate::repositories;
use crate::types::{ConfiguredHook, McpServerDetail, PluginDetail, SkillCatalogEntry};

use super::view::{LinkedEntity, plugin_url};

pub(super) struct Catalog {
    pub(super) plugins: Vec<PluginDetail>,
    pub(super) skills: Vec<SkillCatalogEntry>,
    pub(super) mcp: Vec<McpServerDetail>,
    pub(super) agent_names: HashMap<String, String>,
    pub(super) hooks_by_plugin: HashMap<String, Vec<ConfiguredHook>>,
    pub(super) plugins_by_skill: HashMap<String, Vec<LinkedEntity>>,
    pub(super) plugins_by_mcp: HashMap<String, Vec<LinkedEntity>>,
}

fn to_entity_refs(
    map: repositories::marketplace::plugin_maps::EntityPluginMap,
) -> HashMap<String, Vec<LinkedEntity>> {
    map.into_iter()
        .map(|(entity_id, plugins)| {
            let refs = plugins
                .into_iter()
                .map(|p| LinkedEntity {
                    id: p.0.clone(),
                    name: p.1.clone(),
                    url: plugin_url(&p.0),
                })
                .collect();
            (entity_id, refs)
        })
        .collect()
}

pub(super) fn load_catalog(services_path: &std::path::Path, roles: &[String]) -> Catalog {
    let plugins = repositories::marketplace::plugins::list_plugin_catalog(services_path)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to load plugin catalog");
            Vec::new()
        });
    let skills = repositories::marketplace::plugins::list_skill_catalog(services_path)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to load skill catalog");
            Vec::new()
        });
    let mcp = repositories::mcp::mcp_servers::list_mcp_servers(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to load MCP catalog");
        Vec::new()
    });
    let agent_names = repositories::marketplace::plugins::list_agent_catalog(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|a| (a.id.as_str().to_owned(), a.name))
        .collect();
    let hooks = repositories::marketplace::hooks::list_configured_hooks(services_path, roles)
        .unwrap_or_default();

    let mut hooks_by_plugin: HashMap<String, Vec<ConfiguredHook>> = HashMap::new();
    for hook in hooks {
        hooks_by_plugin
            .entry(hook.plugin_id.as_str().to_owned())
            .or_default()
            .push(hook);
    }

    let (skill_map, _agent_map, mcp_map) =
        repositories::marketplace::plugin_maps::build_entity_plugin_maps(services_path);

    Catalog {
        plugins,
        skills,
        mcp,
        agent_names,
        hooks_by_plugin,
        plugins_by_skill: to_entity_refs(skill_map),
        plugins_by_mcp: to_entity_refs(mcp_map),
    }
}
