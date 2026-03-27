use std::path::Path;

use super::super::types::PlatformPluginConfig;
use super::super::types::PlatformPluginConfigFile;
use crate::error::MarketplaceError;

pub fn _load_authorized_plugin_configs(
    plugins_path: &Path,
    roles: &[String],
) -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let all_plugins = load_all_plugin_configs(plugins_path)?;

    let is_admin = roles.iter().any(|r| r == "admin");
    let mut authorized: Vec<(String, PlatformPluginConfig)> = all_plugins
        .iter()
        .filter(|(_, plugin)| {
            if !plugin.base.enabled {
                return false;
            }
            if is_admin || plugin.roles.is_empty() {
                return true;
            }
            plugin.roles.iter().any(|r| roles.contains(r))
        })
        .cloned()
        .collect();

    let mut seen_ids: std::collections::HashSet<String> =
        authorized.iter().map(|(_, p)| p.base.id.clone()).collect();
    let mut i = 0;
    while i < authorized.len() {
        let deps = authorized[i].1.depends.clone();
        for dep_id in &deps {
            if seen_ids.contains(dep_id) {
                continue;
            }
            if let Some(dep) = all_plugins.iter().find(|(_, p)| p.base.id == *dep_id) {
                if !dep.1.base.enabled {
                    return Err(MarketplaceError::Internal(format!(
                        "Plugin '{}' depends on '{}' which is disabled",
                        authorized[i].1.base.id, dep_id
                    )));
                }
                seen_ids.insert(dep_id.clone());
                authorized.push(dep.clone());
            } else {
                return Err(MarketplaceError::Internal(format!(
                    "Plugin '{}' depends on '{}' which was not found",
                    authorized[i].1.base.id, dep_id
                )));
            }
        }
        i += 1;
    }

    authorized.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(authorized)
}

pub fn load_plugin_configs_by_ids(
    plugins_path: &Path,
    authorized_ids: &std::collections::HashSet<String>,
) -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let all_plugins = load_all_plugin_configs(plugins_path)?;

    let mut authorized: Vec<(String, PlatformPluginConfig)> = all_plugins
        .iter()
        .filter(|(dir_name, plugin)| plugin.base.enabled && authorized_ids.contains(dir_name))
        .cloned()
        .collect();

    let mut seen_ids: std::collections::HashSet<String> =
        authorized.iter().map(|(_, p)| p.base.id.clone()).collect();
    let mut i = 0;
    while i < authorized.len() {
        let deps = authorized[i].1.depends.clone();
        for dep_id in &deps {
            if seen_ids.contains(dep_id) {
                continue;
            }
            if let Some(dep) = all_plugins.iter().find(|(_, p)| p.base.id == *dep_id) {
                if !dep.1.base.enabled {
                    return Err(MarketplaceError::Internal(format!(
                        "Plugin '{}' depends on '{}' which is disabled",
                        authorized[i].1.base.id, dep_id
                    )));
                }
                seen_ids.insert(dep_id.clone());
                authorized.push(dep.clone());
            } else {
                return Err(MarketplaceError::Internal(format!(
                    "Plugin '{}' depends on '{}' which was not found",
                    authorized[i].1.base.id, dep_id
                )));
            }
        }
        i += 1;
    }

    authorized.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(authorized)
}

fn load_all_plugin_configs(
    plugins_path: &Path,
) -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let mut plugins = Vec::new();
    if !plugins_path.exists() {
        return Ok(plugins);
    }

    let mut entries: Vec<_> = std::fs::read_dir(plugins_path)?
        .filter_map(|e| match e {
            Ok(entry) => Some(entry),
            Err(err) => {
                tracing::warn!(error = %err, "Failed to read plugins directory entry");
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
        let content = std::fs::read_to_string(&config_path).map_err(|e| {
            MarketplaceError::Internal(format!("Failed to read {}: {e}", config_path.display()))
        })?;
        let plugin_file: PlatformPluginConfigFile = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                return Err(MarketplaceError::Internal(format!(
                    "Failed to parse {}: {e}",
                    config_path.display()
                )));
            }
        };
        let dir_name = entry.file_name().to_string_lossy().to_string();
        plugins.push((dir_name, plugin_file.plugin));
    }

    Ok(plugins)
}
