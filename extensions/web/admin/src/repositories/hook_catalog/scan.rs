use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::Path;

use super::super::super::types::HookCatalogEntry;
use super::{CATEGORY_CUSTOM, DEFAULT_MATCHER, DEFAULT_VERSION};

#[derive(Debug, Deserialize)]
struct HookConfig {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_version")]
    version: String,
    #[serde(default = "default_true")]
    enabled: bool,
    event: String,
    #[serde(default = "default_matcher")]
    matcher: String,
    #[serde(default)]
    command: String,
    #[serde(default, rename = "async")]
    is_async: bool,
    #[serde(default = "default_custom")]
    category: String,
    #[serde(default)]
    plugins: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    visible_to: Vec<String>,
}

fn default_version() -> String {
    DEFAULT_VERSION.to_owned()
}

fn default_true() -> bool {
    true
}

fn default_matcher() -> String {
    DEFAULT_MATCHER.to_owned()
}

fn default_custom() -> String {
    CATEGORY_CUSTOM.to_owned()
}

pub fn list_file_hooks(services_path: &Path) -> Result<Vec<HookCatalogEntry>, super::HookCatalogError> {
    let hooks_dir = services_path.join("hooks");
    let mut hooks = Vec::new();
    if !hooks_dir.exists() {
        return Ok(hooks);
    }
    for entry in std::fs::read_dir(&hooks_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let dir_name = entry.file_name().to_string_lossy().into_owned();
        let content = std::fs::read_to_string(&config_path)?;
        let config: HookConfig = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(dir = %dir_name, error = %e, "Invalid hook config");
                continue;
            }
        };

        if config.id != dir_name {
            tracing::warn!(
                hook_id = %config.id,
                dir = %dir_name,
                "Hook ID does not match directory name"
            );
            continue;
        }

        let checksum = compute_checksum(&content);
        let now = chrono::Utc::now();

        hooks.push(HookCatalogEntry {
            id: config.id,
            name: config.name,
            description: config.description,
            version: config.version,
            event: config.event,
            matcher: config.matcher,
            command: config.command,
            is_async: config.is_async,
            category: config.category,
            enabled: config.enabled,
            tags: config.tags,
            visible_to: config.visible_to,
            checksum,
            plugins: config.plugins,
            created_at: now,
            updated_at: now,
        });
    }
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hooks)
}

pub fn compute_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
