use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use crate::repositories::plugin_resolvers::resolve_all_plugin_skill_ids;

pub type PluginEntry = Arc<(String, String)>;

pub type EntityPluginMap = HashMap<String, Vec<PluginEntry>>;

#[must_use]
pub fn build_entity_plugin_maps(
    services_path: &Path,
) -> (EntityPluginMap, EntityPluginMap, EntityPluginMap) {
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut skill_map: EntityPluginMap = HashMap::new();
    let mut agent_map: EntityPluginMap = HashMap::new();
    let mut mcp_map: EntityPluginMap = HashMap::new();

    let Ok(plugins) = super::plugin_loader::load_all_plugins() else {
        return (skill_map, agent_map, mcp_map);
    };

    for (_id, wrapper) in plugins {
        let plugin = wrapper.base;
        let plugin_entry: PluginEntry = Arc::new((plugin.id.to_string(), plugin.name.clone()));

        for skill_id in resolve_all_plugin_skill_ids(&plugin, &skills_path, &agents_path) {
            skill_map
                .entry(skill_id)
                .or_default()
                .push(Arc::clone(&plugin_entry));
        }

        for agent_id in &plugin.agents.include {
            agent_map
                .entry(agent_id.clone())
                .or_default()
                .push(Arc::clone(&plugin_entry));
        }

        for mcp_id in &plugin.mcp_servers.include {
            mcp_map
                .entry(mcp_id.clone())
                .or_default()
                .push(Arc::clone(&plugin_entry));
        }
    }

    (skill_map, agent_map, mcp_map)
}
