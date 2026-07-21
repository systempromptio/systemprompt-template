use std::path::PathBuf;

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::PluginConfigFile;

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
    serde_yaml::from_str::<PluginConfigFile>(&content)
        .map_err(|e| MarketplaceError::Internal(format!("Failed to parse {}: {e}", path.display())))
}

pub(crate) fn load_all_plugins() -> Result<Vec<(String, PlatformPluginConfig)>, MarketplaceError> {
    Ok(load_all_plugins_with_paths()?
        .into_iter()
        .map(|(id, cfg, _path)| (id, cfg))
        .collect())
}

pub(crate) fn load_all_plugins_with_paths()
-> Result<Vec<(String, PlatformPluginConfig, String)>, MarketplaceError> {
    let dir = plugins_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(&dir).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to read {}: {e}", dir.display()))
    })?;

    let services_dir = dir.parent().map(std::path::Path::to_path_buf);

    let mut out: Vec<(String, PlatformPluginConfig, String)> = Vec::new();
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(error = %e, "skipped unreadable plugin entry");
                continue;
            },
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
                let source_path = services_dir
                    .as_ref()
                    .and_then(|sd| config_path.strip_prefix(sd).ok())
                    .and_then(|p| p.to_str())
                    .map_or_else(
                        || config_path.display().to_string(),
                        |s| format!("services/{s}"),
                    );
                out.push((id, PlatformPluginConfig::from_base(pf.plugin), source_path));
            },
            Err(e) => {
                tracing::warn!(path = %config_path.display(), error = %e, "skipped invalid plugin config");
            },
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}
