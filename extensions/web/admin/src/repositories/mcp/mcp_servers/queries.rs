use std::path::Path;

use crate::types::McpServerDetail;
use systemprompt::identifiers::McpServerId;
use systemprompt_web_shared::error::MarketplaceError;

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
        let rel_source = path
            // Why: strip_prefix returns Err when the path isn't under services_path
            // (e.g. an absolute legacy entry); the fallback constructs a synthetic
            // path string instead of bailing.
            .strip_prefix(services_path)
            .ok()
            .and_then(|p| p.to_str())
            .map_or_else(
                || {
                    format!(
                        "services/mcp/{}",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                    )
                },
                |s| format!("services/{s}"),
            );
        if let Some(mcp_map) = config.get("mcp_servers").and_then(|m| m.as_mapping()) {
            for (key, val) in mcp_map {
                if let Some(server_id) = key.as_str()
                    && let Ok(id) = McpServerId::try_new(server_id)
                {
                    servers.push(parse_server_detail(id, val, rel_source.clone()));
                }
            }
        }
    }
    servers.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(servers)
}

fn parse_server_detail(
    server_id: McpServerId,
    val: &serde_yaml::Value,
    source_path: String,
) -> McpServerDetail {
    let binary = val
        .get("binary")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, str::to_owned);
    let server_type = val
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or(if binary.is_empty() {
            "external"
        } else {
            "internal"
        })
        .to_owned();
    let package_name = val
        .get("package")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, str::to_owned);
    let port = val
        .get("port")
        .and_then(serde_yaml::Value::as_u64)
        .and_then(|p| u16::try_from(p).ok())
        .unwrap_or(DEFAULT_MCP_PORT);
    let endpoint = val
        .get("endpoint")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, str::to_owned);
    let description = val
        .get("description")
        .and_then(|v| v.as_str())
        .map_or_else(String::new, str::to_owned);
    let enabled = val
        .get("enabled")
        .and_then(serde_yaml::Value::as_bool)
        .unwrap_or(true);
    let oauth = val.get("oauth");
    let oauth_required = oauth
        .and_then(|o| o.get("required"))
        .and_then(serde_yaml::Value::as_bool)
        .unwrap_or(false);
    let oauth_scopes: Vec<String> = oauth
        .and_then(|o| o.get("scopes"))
        .and_then(|v| v.as_sequence())
        .map_or_else(Vec::new, |s| {
            s.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        });
    let oauth_audience = oauth
        .and_then(|o| o.get("audience"))
        .and_then(|v| v.as_str())
        .map_or_else(String::new, str::to_owned);

    McpServerDetail {
        id: server_id,
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
        source_path,
    }
}
