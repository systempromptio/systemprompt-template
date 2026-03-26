use std::fmt::Write;
use std::path::Path;

use super::super::types::{CreateMcpRequest, McpServerDetail, UpdateMcpRequest};
use super::mcp_servers_yaml::find_mcp_file;

const DEFAULT_MCP_PORT: u16 = 5000;

pub fn list_mcp_servers(services_path: &Path) -> Result<Vec<McpServerDetail>, anyhow::Error> {
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
                    let removable = val
                        .get("removable")
                        .and_then(serde_yaml::Value::as_bool)
                        .unwrap_or(true);
                    servers.push(McpServerDetail {
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
                        removable,
                    });
                }
            }
        }
    }
    servers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(servers)
}

pub fn get_mcp_server(
    services_path: &Path,
    server_id: &str,
) -> Result<Option<McpServerDetail>, anyhow::Error> {
    let servers = list_mcp_servers(services_path)?;
    Ok(servers.into_iter().find(|s| s.id == server_id))
}

pub fn create_mcp_server(
    services_path: &Path,
    req: &CreateMcpRequest,
) -> Result<McpServerDetail, anyhow::Error> {
    use anyhow::Context;
    let mcp_dir = services_path.join("mcp");
    std::fs::create_dir_all(&mcp_dir)?;
    let file_path = mcp_dir.join(format!("{}.yaml", req.id));
    if file_path.exists() {
        anyhow::bail!("MCP server '{}' already exists", req.id);
    }
    let server_type = if req.server_type.is_empty() {
        if req.binary.is_empty() {
            "external"
        } else {
            "internal"
        }
    } else {
        &req.server_type
    };
    let endpoint = if req.endpoint.is_empty() && server_type == "internal" {
        format!("http://localhost:8080/api/v1/mcp/{}/mcp", req.id)
    } else if req.endpoint.is_empty() && server_type == "external" {
        anyhow::bail!("External MCP server '{}' requires an endpoint URL", req.id);
    } else {
        req.endpoint.clone()
    };
    let mut yaml_content = format!(
        "mcp_servers:\n  {}:\n    type: {}\n    binary: \"{}\"\n    package: \"{}\"\n    port: {}\n    endpoint: \"{}\"\n    enabled: {}\n    display_in_web: true\n    description: \"{}\"\n",
        req.id, server_type, req.binary, req.package_name, req.port, endpoint, req.enabled, req.description,
    );
    if req.oauth_required {
        let scopes_yaml: String = req
            .oauth_scopes
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            yaml_content,
            "\n    oauth:\n      required: true\n      scopes: [{scopes_yaml}]\n      audience: \"{}\"\n      client_id: null\n",
            req.oauth_audience
        ).expect("write to String cannot fail");
    }
    std::fs::write(&file_path, &yaml_content)
        .with_context(|| format!("Failed to write: {}", file_path.display()))?;
    Ok(McpServerDetail {
        id: req.id.clone(),
        server_type: server_type.to_string(),
        binary: req.binary.clone(),
        package_name: req.package_name.clone(),
        port: req.port,
        endpoint,
        description: req.description.clone(),
        enabled: req.enabled,
        oauth_required: req.oauth_required,
        oauth_scopes: req.oauth_scopes.clone(),
        oauth_audience: req.oauth_audience.clone(),
        removable: true,
    })
}

pub fn update_mcp_server(
    services_path: &Path,
    server_id: &str,
    req: &UpdateMcpRequest,
) -> Result<Option<McpServerDetail>, anyhow::Error> {
    use anyhow::Context;
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    if let Some(sv) = doc
        .get_mut("mcp_servers")
        .and_then(|m| m.get_mut(server_id))
    {
        if let Some(ref t) = req.server_type {
            sv["type"] = serde_yaml::Value::String(t.clone());
        }
        if let Some(ref b) = req.binary {
            sv["binary"] = serde_yaml::Value::String(b.clone());
        }
        if let Some(ref p) = req.package_name {
            sv["package"] = serde_yaml::Value::String(p.clone());
        }
        if let Some(p) = req.port {
            sv["port"] = serde_yaml::Value::Number(serde_yaml::Number::from(p));
        }
        if let Some(ref e) = req.endpoint {
            sv["endpoint"] = serde_yaml::Value::String(e.clone());
        }
        if let Some(ref d) = req.description {
            sv["description"] = serde_yaml::Value::String(d.clone());
        }
        if let Some(e) = req.enabled {
            sv["enabled"] = serde_yaml::Value::Bool(e);
        }
        if let Some(oauth_val) = sv.get_mut("oauth") {
            if let Some(r) = req.oauth_required {
                oauth_val["required"] = serde_yaml::Value::Bool(r);
            }
            if let Some(ref s) = req.oauth_scopes {
                oauth_val["scopes"] = serde_yaml::Value::Sequence(
                    s.iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                );
            }
            if let Some(ref a) = req.oauth_audience {
                oauth_val["audience"] = serde_yaml::Value::String(a.clone());
            }
        }
    }
    std::fs::write(&file_path, serde_yaml::to_string(&doc)?)
        .with_context(|| format!("Failed to write: {}", file_path.display()))?;
    get_mcp_server(services_path, server_id)
}

pub fn delete_mcp_server(services_path: &Path, server_id: &str) -> Result<bool, anyhow::Error> {
    if let Ok(Some(server)) = get_mcp_server(services_path, server_id) {
        if !server.removable {
            anyhow::bail!(
                "MCP server '{server_id}' is a system default and cannot be deleted"
            );
        }
    }
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(false);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let count = doc
        .get("mcp_servers")
        .and_then(|m| m.as_mapping())
        .map_or(0, serde_yaml::Mapping::len);
    if count <= 1 {
        std::fs::remove_file(&file_path)?;
    } else {
        let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if let Some(s) = doc.get_mut("mcp_servers").and_then(|m| m.as_mapping_mut()) {
            s.remove(serde_yaml::Value::String(server_id.to_string()));
        }
        std::fs::write(&file_path, serde_yaml::to_string(&doc)?)?;
    }
    Ok(true)
}
