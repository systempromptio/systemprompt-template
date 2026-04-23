use std::path::PathBuf;

use systemprompt::models::{PluginConfig, PluginConfigFile, ProfileBootstrap};

use crate::types::PlatformPluginConfig;
use systemprompt_web_shared::error::MarketplaceError;

fn plugins_dir() -> Result<PathBuf, MarketplaceError> {
    let profile = ProfileBootstrap::get().map_err(|e| {
        tracing::error!(error = %e, "Failed to get profile bootstrap");
        MarketplaceError::Internal(format!("Failed to get profile bootstrap: {e}"))
    })?;
    Ok(PathBuf::from(&profile.paths.services).join("plugins"))
}

fn parse_plugin_config_file(path: &std::path::Path) -> Result<PluginConfigFile, MarketplaceError> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to read {}: {e}", path.display()))
    })?;
    serde_yaml::from_str::<PluginConfigFile>(&content).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to parse {}: {e}", path.display()))
    })
}

pub fn load_all_plugins() -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    let dir = plugins_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&dir).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to read {}: {e}", dir.display()))
    })?;

    let mut out: Vec<(String, PlatformPluginConfig)> = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "skipped unreadable plugin entry");
                continue;
            }
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        match parse_plugin_config_file(&config_path) {
            Ok(pf) => {
                let id = pf.plugin.id.to_string();
                out.push((id, PlatformPluginConfig::from_base(pf.plugin)));
            }
            Err(e) => {
                tracing::warn!(path = %config_path.display(), error = %e, "skipped invalid plugin config");
            }
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

pub fn find_plugin(plugin_id: &str) -> Result<Option<PluginConfig>, MarketplaceError> {
    Ok(load_all_plugins()?
        .into_iter()
        .find(|(id, _)| id == plugin_id)
        .map(|(_, cfg)| cfg.base))
}
