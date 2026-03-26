use std::collections::HashMap;
use std::path::Path;

use super::super::types::{PluginOnboardingConfig, PluginOverview};
use super::plugin_hook_resolvers::resolve_plugin_hooks;
use super::plugin_resolvers::{
    resolve_all_plugin_skill_ids, resolve_plugin_agents, resolve_plugin_skills,
};

pub fn list_all_skill_ids(services_path: &Path) -> Result<Vec<String>, anyhow::Error> {
    let skills_path = services_path.join("skills");
    let mut ids = Vec::new();
    if !skills_path.exists() {
        return Ok(ids);
    }
    for entry in std::fs::read_dir(&skills_path)? {
        let entry = entry?;
        if entry.path().is_dir() {
            ids.push(entry.file_name().to_string_lossy().to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

pub fn get_plugin_skill_ids(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Vec<String>, anyhow::Error> {
    use anyhow::Context;
    use systemprompt::models::PluginConfigFile;

    let config_path = services_path
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Plugin config not found: {}", config_path.display()))?;

    let plugin_file: PluginConfigFile = serde_yaml::from_str(&content)
        .with_context(|| format!("Invalid plugin config: {}", config_path.display()))?;

    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    Ok(resolve_all_plugin_skill_ids(
        &plugin_file.plugin,
        &skills_path,
        &agents_path,
    ))
}

pub fn update_plugin_skills(
    services_path: &Path,
    plugin_id: &str,
    skill_ids: &[String],
) -> Result<(), anyhow::Error> {
    use anyhow::Context;

    let config_path = services_path
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");

    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("Plugin config not found: {}", config_path.display()))?;

    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| format!("Invalid YAML: {}", config_path.display()))?;

    if let Some(plugin) = doc.get_mut("plugin") {
        if let Some(skills) = plugin.get_mut("skills") {
            skills["source"] = serde_yaml::Value::String("explicit".to_string());
            let include_seq: Vec<serde_yaml::Value> = skill_ids
                .iter()
                .map(|s| serde_yaml::Value::String(s.clone()))
                .collect();
            skills["include"] = serde_yaml::Value::Sequence(include_seq);
        }
    }

    let yaml_str = serde_yaml::to_string(&doc).with_context(|| "Failed to serialize YAML")?;
    std::fs::write(&config_path, yaml_str)
        .with_context(|| format!("Failed to write: {}", config_path.display()))?;

    Ok(())
}

pub struct MarketplaceCounts {
    pub total_plugins: usize,
    pub total_skills: usize,
    pub agents_count: usize,
    pub mcp_count: usize,
}

pub fn count_marketplace_items(
    services_path: &Path,
    roles: &[String],
) -> Result<MarketplaceCounts, anyhow::Error> {
    use super::super::types::PlatformPluginConfigFile;

    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut counts = MarketplaceCounts {
        total_plugins: 0,
        total_skills: 0,
        agents_count: 0,
        mcp_count: 0,
    };

    if !plugins_path.exists() {
        return Ok(counts);
    }

    for entry in std::fs::read_dir(&plugins_path)? {
        let entry = entry?;
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
        let Ok(plugin_file): Result<PlatformPluginConfigFile, _> = serde_yaml::from_str(&content)
        else {
            continue;
        };
        let plugin = plugin_file.plugin;
        if !plugin.base.enabled {
            continue;
        }
        let is_admin = roles.iter().any(|r| r == "admin");
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        counts.total_plugins += 1;
        counts.total_skills +=
            resolve_all_plugin_skill_ids(&plugin.base, &skills_path, &agents_path).len();
        counts.agents_count += plugin.base.agents.include.len();
        counts.mcp_count += plugin.base.mcp_servers.len();
    }

    Ok(counts)
}

pub fn list_plugins_for_roles(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<PluginOverview>, anyhow::Error> {
    list_plugins_for_roles_full(services_path, roles, &HashMap::new(), &HashMap::new())
}

#[allow(clippy::implicit_hasher)]
pub fn list_plugins_for_roles_full(
    services_path: &Path,
    roles: &[String],
    enabled_map: &HashMap<String, bool>,
    hook_enabled_map: &HashMap<String, bool>,
) -> Result<Vec<PluginOverview>, anyhow::Error> {
    use super::super::types::PlatformPluginConfigFile;
    use anyhow::Context;

    let plugins_path = services_path.join("plugins");
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");

    let mut overviews = Vec::new();

    if !plugins_path.exists() {
        return Ok(overviews);
    }

    let mut entries: Vec<_> = std::fs::read_dir(&plugins_path)?
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                tracing::warn!(error = %err, "Failed to read plugin directory entry");
                None
            }
        })
        .collect();
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read {}", config_path.display()))?;

        let plugin_file: PlatformPluginConfigFile = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let plugin = plugin_file.plugin;

        let is_admin = roles.iter().any(|r| r == "admin");

        if !plugin.base.enabled && !is_admin {
            continue;
        }

        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }

        let skill_infos =
            resolve_plugin_skills(&plugin.base, &skills_path, &agents_path, enabled_map);
        let agent_infos = resolve_plugin_agents(&plugin.base, &agents_path);
        let hook_infos = resolve_plugin_hooks(&config_path, &plugin.base.id, hook_enabled_map);

        overviews.push(PluginOverview {
            id: plugin.base.id,
            name: plugin.base.name,
            description: plugin.base.description,
            enabled: plugin.base.enabled,
            skills: skill_infos,
            agents: agent_infos,
            mcp_servers: plugin.base.mcp_servers,
            hooks: hook_infos,
            depends: plugin.depends,
        });
    }

    Ok(overviews)
}

#[must_use]
pub fn load_plugin_onboarding_configs() -> HashMap<String, PluginOnboardingConfig> {
    use super::super::types::PlatformPluginConfigFile;
    use systemprompt::models::ProfileBootstrap;

    let plugins_path = if let Ok(profile) = ProfileBootstrap::get() {
        std::path::PathBuf::from(&profile.paths.services).join("plugins")
    } else {
        let cwd = std::env::current_dir().unwrap_or_default();
        cwd.join("services").join("plugins")
    };
    let mut configs = HashMap::new();

    let Ok(entries) = std::fs::read_dir(&plugins_path) else {
        return configs;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(plugin_file): Result<PlatformPluginConfigFile, _> = serde_yaml::from_str(&content)
        else {
            continue;
        };
        if let Some(onboarding) = plugin_file.plugin.onboarding {
            configs.insert(plugin_file.plugin.base.id, onboarding);
        }
    }

    configs
}
