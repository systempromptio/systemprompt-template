use std::path::Path;

use super::super::types::McpServerDetail;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, thiserror::Error)]
pub enum McpConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("YAML must contain mcp_servers.{0} key")]
    MissingServerKey(String),
    #[error("{0}")]
    Marketplace(#[from] MarketplaceError),
}

pub fn find_mcp_server_raw_yaml(
    services_path: &Path,
    server_id: &str,
) -> Result<Option<(String, String)>, McpConfigError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown.yaml")
        .to_string();
    Ok(Some((content, file_name)))
}

pub fn update_mcp_server_raw_yaml(
    services_path: &Path,
    server_id: &str,
    yaml_content: &str,
) -> Result<Option<McpServerDetail>, McpConfigError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(None);
    };
    let doc: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
    let has_server = doc
        .get("mcp_servers")
        .and_then(|m| m.get(server_id))
        .is_some();
    if !has_server {
        return Err(McpConfigError::MissingServerKey(server_id.to_string()));
    }
    std::fs::write(&file_path, yaml_content)?;
    Ok(super::mcp_servers::get_mcp_server(services_path, server_id)?)
}

pub(crate) fn find_mcp_file(
    mcp_dir: &Path,
    server_id: &str,
) -> Result<Option<std::path::PathBuf>, McpConfigError> {
    if !mcp_dir.exists() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(mcp_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if config
            .get("mcp_servers")
            .and_then(|m| m.get(server_id))
            .is_some()
        {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
