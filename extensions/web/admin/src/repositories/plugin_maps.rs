use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use super::plugin_resolvers::resolve_all_plugin_skill_ids;

pub type PluginEntry = Arc<(String, String)>;

pub type EntityPluginMap = HashMap<String, Vec<PluginEntry>>;

#[must_use]
pub fn build_entity_plugin_maps(
    services_path: &Path,
) -> (EntityPluginMap, EntityPluginMap, EntityPluginMap) {
    use systemprompt::models::PluginConfigFile;

    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut skill_map: EntityPluginMap = HashMap::new();
    let mut agent_map: EntityPluginMap = HashMap::new();
    let mut mcp_map: EntityPluginMap = HashMap::new();

    if !plugins_path.exists() {
        return (skill_map, agent_map, mcp_map);
    }

    let Ok(entries) = std::fs::read_dir(&plugins_path) else {
        return (skill_map, agent_map, mcp_map);
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(plugin_file): Result<PluginConfigFile, _> = serde_yaml::from_str(&content) else {
            continue;
        };
        let plugin = plugin_file.plugin;
        let plugin_entry: PluginEntry = Arc::new((plugin.id.clone(), plugin.name.clone()));

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

        for mcp_id in &plugin.mcp_servers {
            mcp_map
                .entry(mcp_id.clone())
                .or_default()
                .push(Arc::clone(&plugin_entry));
        }
    }

    (skill_map, agent_map, mcp_map)
}
