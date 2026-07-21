use std::path::Path;

use serde::Serialize;

use crate::repositories;

/// A generic labelled entity reference used by dropdowns/lookup tables in the
/// unified access-control UI (mcp servers, plugins, agents, marketplaces).
#[derive(Debug, Serialize)]
pub(super) struct EntityRef {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) description: String,
}

/// Gateway routes have a `provider` instead of a `description`.
#[derive(Debug, Serialize)]
pub(super) struct RouteRef {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) provider: String,
}

/// Lightweight list of every entity that can have an ACL rule attached, used
/// by the unified access-control UI to populate dropdowns and lookup tables.
#[derive(Debug, Serialize)]
pub(super) struct EntityCatalogue {
    pub(super) gateway_routes: Vec<RouteRef>,
    pub(super) mcp_servers: Vec<EntityRef>,
    pub(super) plugins: Vec<EntityRef>,
    pub(super) agents: Vec<EntityRef>,
    pub(super) marketplaces: Vec<EntityRef>,
}

pub(super) fn build_entity_catalogue(services_path: &Path) -> EntityCatalogue {
    EntityCatalogue {
        gateway_routes: build_gateway_routes(services_path),
        mcp_servers: build_mcp_servers(services_path),
        plugins: build_plugins(services_path),
        agents: build_agents(services_path),
        marketplaces: build_marketplaces(),
    }
}

fn build_gateway_routes(services_path: &Path) -> Vec<RouteRef> {
    let Some(parent) = services_path.parent() else {
        return Vec::new();
    };
    let candidates = [
        parent.join("profile.yaml"),
        services_path.join("../.systemprompt/profiles/local/profile.yaml"),
    ];
    for path in &candidates {
        if path.exists()
            && let Ok(cfg) = repositories::governance::gateway::get_gateway_config(path)
        {
            return cfg
                .routes
                .into_iter()
                .map(|r| RouteRef {
                    id: r.id,
                    label: r.model_pattern,
                    provider: r.provider,
                })
                .collect();
        }
    }
    Vec::new()
}

fn build_mcp_servers(services_path: &Path) -> Vec<EntityRef> {
    repositories::mcp::mcp_servers::list_mcp_servers(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|s| EntityRef {
            id: s.id.as_str().to_owned(),
            label: s.id.as_str().to_owned(),
            description: s.description,
        })
        .collect()
}

fn build_plugins(services_path: &Path) -> Vec<EntityRef> {
    let admin_roles = vec!["admin".to_owned()];
    repositories::marketplace::plugins::list_plugins_for_roles(services_path, &admin_roles)
        .unwrap_or_default()
        .into_iter()
        .map(|p| EntityRef {
            id: p.id,
            label: p.name,
            description: p.description,
        })
        .collect()
}

fn build_marketplaces() -> Vec<EntityRef> {
    crate::services::marketplaces::load_marketplaces()
        .into_iter()
        .map(|mp| EntityRef {
            id: mp.id.as_str().to_owned(),
            label: mp.name,
            description: mp.description,
        })
        .collect()
}

fn build_agents(services_path: &Path) -> Vec<EntityRef> {
    repositories::governance::agents::list_agents(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|a| EntityRef {
            id: a.id.as_str().to_owned(),
            label: a.name,
            description: a.description,
        })
        .collect()
}
