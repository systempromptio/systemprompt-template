use std::path::Path;

use super::queries::find_mcp_server;
use crate::types::{
    CreateMcpRequest, McpServerDetail, SERVER_TYPE_EXTERNAL, SERVER_TYPE_INTERNAL, UpdateMcpRequest,
};
use systemprompt_web_shared::error::MarketplaceError;

pub fn create_mcp_server(
    services_path: &Path,
    req: CreateMcpRequest,
) -> Result<McpServerDetail, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    std::fs::create_dir_all(&mcp_dir)?;
    let file_path = mcp_dir.join(format!("{}.yaml", req.id));
    if file_path.exists() {
        return Err(MarketplaceError::Internal(format!(
            "MCP server '{}' already exists",
            req.id
        )));
    }
    let server_type = if req.server_type.is_empty() {
        if req.binary.is_empty() {
            "external".to_owned()
        } else {
            "internal".to_owned()
        }
    } else {
        req.server_type
    };
    let endpoint = if req.endpoint.is_empty() && server_type == SERVER_TYPE_INTERNAL {
        format!("http://localhost:8080/api/v1/mcp/{}/mcp", req.id)
    } else if req.endpoint.is_empty() && server_type == SERVER_TYPE_EXTERNAL {
        return Err(MarketplaceError::Internal(format!(
            "External MCP server '{}' requires an endpoint URL",
            req.id
        )));
    } else {
        req.endpoint
    };
    let mut yaml_content = format!(
        "mcp_servers:\n  {}:\n    type: {}\n    binary: \"{}\"\n    package: \"{}\"\n    port: {}\n    endpoint: \"{}\"\n    enabled: {}\n    display_in_web: true\n    description: \"{}\"\n",
        req.id,
        server_type,
        req.binary,
        req.package_name,
        req.port,
        endpoint,
        req.enabled,
        req.description,
    );
    if req.oauth_required {
        let scopes_yaml: String = req
            .oauth_scopes
            .iter()
            .map(|s| format!("\"{s}\""))
            .collect::<Vec<_>>()
            .join(", ");
        yaml_content.push_str(&format!(
            "\n    oauth:\n      required: true\n      scopes: [{scopes_yaml}]\n      audience: \"{}\"\n      client_id: null\n",
            req.oauth_audience
        ));
    }
    std::fs::write(&file_path, &yaml_content).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to write: {}: {e}", file_path.display()))
    })?;
    Ok(McpServerDetail {
        id: req.id,
        server_type,
        binary: req.binary,
        package_name: req.package_name,
        port: req.port,
        endpoint,
        description: req.description,
        enabled: req.enabled,
        oauth_required: req.oauth_required,
        oauth_scopes: req.oauth_scopes,
        oauth_audience: req.oauth_audience,
        removable: true,
        source_path: String::new(),
    })
}

pub fn update_mcp_server(
    services_path: &Path,
    server_id: &str,
    req: UpdateMcpRequest,
) -> Result<Option<McpServerDetail>, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = super::find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    if let Some(sv) = doc
        .get_mut("mcp_servers")
        .and_then(|m| m.get_mut(server_id))
    {
        apply_update_fields(sv, req);
    }
    std::fs::write(&file_path, serde_yaml::to_string(&doc)?).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to write: {}: {e}", file_path.display()))
    })?;
    find_mcp_server(services_path, server_id)
}

fn apply_update_fields(sv: &mut serde_yaml::Value, req: UpdateMcpRequest) {
    if let Some(t) = req.server_type {
        sv["type"] = serde_yaml::Value::String(t);
    }
    if let Some(b) = req.binary {
        sv["binary"] = serde_yaml::Value::String(b);
    }
    if let Some(p) = req.package_name {
        sv["package"] = serde_yaml::Value::String(p);
    }
    if let Some(p) = req.port {
        sv["port"] = serde_yaml::Value::Number(serde_yaml::Number::from(p));
    }
    if let Some(e) = req.endpoint {
        sv["endpoint"] = serde_yaml::Value::String(e);
    }
    if let Some(d) = req.description {
        sv["description"] = serde_yaml::Value::String(d);
    }
    if let Some(e) = req.enabled {
        sv["enabled"] = serde_yaml::Value::Bool(e);
    }
    if let Some(oauth_val) = sv.get_mut("oauth") {
        if let Some(r) = req.oauth_required {
            oauth_val["required"] = serde_yaml::Value::Bool(r);
        }
        if let Some(s) = req.oauth_scopes {
            oauth_val["scopes"] =
                serde_yaml::Value::Sequence(s.into_iter().map(serde_yaml::Value::String).collect());
        }
        if let Some(a) = req.oauth_audience {
            oauth_val["audience"] = serde_yaml::Value::String(a);
        }
    }
}

pub fn delete_mcp_server(services_path: &Path, server_id: &str) -> Result<bool, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = super::find_mcp_file(&mcp_dir, server_id)? else {
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
            s.remove(serde_yaml::Value::String(server_id.to_owned()));
        }
        std::fs::write(&file_path, serde_yaml::to_string(&doc)?)?;
    }
    Ok(true)
}

pub fn update_mcp_server_raw_yaml(
    services_path: &Path,
    server_id: &str,
    yaml_content: &str,
) -> Result<Option<McpServerDetail>, MarketplaceError> {
    let mcp_dir = services_path.join("mcp");
    let Some(file_path) = super::find_mcp_file(&mcp_dir, server_id)? else {
        return Ok(None);
    };
    let doc: serde_yaml::Value = serde_yaml::from_str(yaml_content)
        .map_err(|e| MarketplaceError::Internal(format!("Invalid YAML syntax: {e}")))?;
    let has_server = doc
        .get("mcp_servers")
        .and_then(|m| m.get(server_id))
        .is_some();
    if !has_server {
        return Err(MarketplaceError::Internal(format!(
            "YAML must contain mcp_servers.{server_id} key"
        )));
    }
    std::fs::write(&file_path, yaml_content).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to write: {}: {e}", file_path.display()))
    })?;
    find_mcp_server(services_path, server_id)
}
