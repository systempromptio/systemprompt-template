use std::path::Path;

use serde_json::json;

use crate::repositories;

/// Lightweight list of every entity that can have an ACL rule attached, used
/// by the unified access-control UI to populate dropdowns and lookup tables.
pub(super) fn build_entity_catalogue(services_path: &Path) -> serde_json::Value {
    let gateway_routes = build_gateway_routes(services_path);
    let mcp_servers = build_mcp_servers(services_path);
    let plugins = build_plugins(services_path);
    let agents = build_agents(services_path);
    json!({
        "gateway_routes": gateway_routes,
        "mcp_servers": mcp_servers,
        "plugins": plugins,
        "agents": agents,
    })
}

fn build_gateway_routes(services_path: &Path) -> Vec<serde_json::Value> {
    let Some(parent) = services_path.parent() else {
        return Vec::new();
    };
    let candidates = [
        parent.join("profile.yaml"),
        services_path.join("../.systemprompt/profiles/local/profile.yaml"),
    ];
    for path in &candidates {
        if path.exists() {
            if let Ok(cfg) = repositories::get_gateway_config(path) {
                return cfg
                    .routes
                    .into_iter()
                    .map(|r| {
                        json!({
                            "id": r.id,
                            "label": r.model_pattern,
                            "provider": r.provider,
                        })
                    })
                    .collect();
            }
        }
    }
    Vec::new()
}

fn build_mcp_servers(services_path: &Path) -> Vec<serde_json::Value> {
    repositories::mcp_servers::list_mcp_servers(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|s| {
            json!({
                "id": s.id.as_str(),
                "label": s.id.as_str(),
                "description": s.description,
            })
        })
        .collect()
}

fn build_plugins(services_path: &Path) -> Vec<serde_json::Value> {
    let admin_roles = vec!["admin".to_string()];
    repositories::list_plugins_for_roles(services_path, &admin_roles)
        .unwrap_or_default()
        .into_iter()
        .map(|p| {
            json!({
                "id": p.id,
                "label": p.name,
                "description": p.description,
            })
        })
        .collect()
}

fn build_agents(services_path: &Path) -> Vec<serde_json::Value> {
    repositories::list_agents(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id.as_str(),
                "label": a.name,
                "description": a.description,
            })
        })
        .collect()
}
