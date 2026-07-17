use std::path::Path;

use systemprompt::identifiers::PluginId;
use systemprompt::models::{DiskHookConfig, HOOK_CONFIG_FILENAME};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::ConfiguredHook;

pub fn list_configured_hooks(
    services_path: &Path,
    roles: &[String],
) -> Result<Vec<ConfiguredHook>, MarketplaceError> {
    use crate::types::ROLE_ADMIN;
    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);

    let hooks_dir = services_path.join("hooks");
    if !hooks_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    let read = std::fs::read_dir(&hooks_dir)?;
    for entry in read {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let config_path = path.join(HOOK_CONFIG_FILENAME);
        if !config_path.exists() {
            continue;
        }

        let dir_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_owned();

        let Ok(text) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let config: DiskHookConfig = match serde_yaml::from_str(&text) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(path = %config_path.display(), error = %e, "parse hook config");
                continue;
            },
        };

        if !config.enabled && !is_admin {
            continue;
        }
        if !is_admin
            && !config.visible_to.is_empty()
            && !config.visible_to.iter().any(|r| roles.contains(r))
        {
            continue;
        }

        let id_str = if config.id.as_str().is_empty() {
            dir_name
        } else {
            config.id.as_str().to_owned()
        };

        out.push(ConfiguredHook {
            id: id_str.clone(),
            plugin_id: PluginId::new(id_str),
            event: config.event.as_str().to_owned(),
            matcher: config.matcher.clone(),
            command: config.command.clone(),
            is_async: config.is_async,
            timeout_ms: None,
        });
    }

    Ok(out)
}
