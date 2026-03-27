use std::path::Path;



use super::super::super::types::McpServerDetail;
use crate::error::MarketplaceError;

const DEFAULT_MCP_PORT: u16 = 5000;

pub fn list_mcp_servers(services_path: &Path) -> Result<Vec<McpServerDetail>, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    let mut servers = Vec::new();
    if !mcp_dir.exists() {
        return Ok(servers);
    }
    for entry in std::fs::read_dir(&mcp_dir)? {
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
        if let Some(mcp_map) = config.get("mcp_servers").and_then(|m| m.as_mapping()) {
            for (key, val) in mcp_map {
                if let Some(server_id) = key.as_str() {
                    servers.push(parse_server_detail(server_id, val));
                }
            }
        }
    }
    servers.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(servers)
}

fn parse_server_detail(server_id: &str, val: &serde_yaml::Value) -> McpServerDetail {
    let binary = val
        .get("binary")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, ToString::to_string);
    let server_type = val
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or(if binary.is_empty() {
            "external"
        } else {
            "internal"
        })
        .to_string();
    let package_name = val
        .get("package")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, ToString::to_string);
    let port = val
        .get("port")
        .and_then(serde_yaml::Value::as_u64)
        .and_then(|p| u16::try_from(p).ok())
        .unwrap_or(DEFAULT_MCP_PORT);
    let endpoint = val
        .get("endpoint")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, ToString::to_string);
    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, ToString::to_string);
    let enabled = val
        .get("enabled")
        .and_then(serde_yaml::Value::as_bool)
        .unwrap_or(true);
    let oauth = val.get("oauth");
    let oauth_required = oauth
        .and_then(|o| o.get("required"))
        .and_then(serde_yaml::Value::as_bool)
        .unwrap_or(false);
    let oauth_scopes: Vec<String> = match oauth
        .and_then(|o| o.get("scopes"))
        .and_then(|v| v.as_sequence())
    {
        Some(s) => s
            .iter()
            .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
            .collect(),
        None => Vec::new(),
    };
    let oauth_audience = oauth
        .and_then(|o| o.get("audience"))
        .and_then(|v| v.as_str())
        .map_or_else(String::new, ToString::to_string);

    McpServerDetail {
        id: server_id.to_string(),
        server_type,
        binary,
        package_name,
        port,
        endpoint,
        description,
        enabled,
        oauth_required,
        oauth_scopes,
        oauth_audience,
        removable: true,
    }
}

pub fn find_mcp_server(
    services_path: &Path,
    server_id: &str,
) -> Result<Option<McpServerDetail>, MarketplaceError> {
    let servers = list_mcp_servers(services_path)?;
    Ok(servers.into_iter().find(|s| s.id.as_str() == server_id))
}

pub fn find_mcp_server_raw_yaml(
    services_path: &Path,
    server_id: &str,
) -> Result<Option<(String, String)>, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = super::find_mcp_file(&mcp_dir, server_id)? else {
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
