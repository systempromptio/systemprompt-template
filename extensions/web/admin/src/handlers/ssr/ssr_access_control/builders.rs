//! Enumerates every entity an access-control rule can attach to.
//!
//! Sources are read from the services config tree rather than the database
//! because the catalog is bootstrap state, not runtime state.

use std::path::Path;

use serde::Serialize;

use crate::repositories;

/// A generic labelled entity reference used by dropdowns/lookup tables in the
/// unified access-control UI (mcp servers, plugins, agents, marketplaces).
#[derive(Debug, Serialize)]
pub(super) struct EntityOption {
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
    pub(super) mcp_servers: Vec<EntityOption>,
    pub(super) plugins: Vec<EntityOption>,
    pub(super) agents: Vec<EntityOption>,
    pub(super) marketplaces: Vec<EntityOption>,
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
    // An unparseable profile.yaml would otherwise leave no trace at all: the
    // catalogue renders "No entities of this type configured", which is
    // indistinguishable from a gateway that genuinely has no routes.
    for path in &candidates {
        if path.exists()
            && let Ok(cfg) = repositories::config::gateway::get_gateway_config(path)
                .inspect_err(|e| tracing::warn!(error = %e, path = %path.display(), "gateway config unreadable; routes omitted from the access-control catalogue"))
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

fn build_mcp_servers(services_path: &Path) -> Vec<EntityOption> {
    repositories::mcp::mcp_servers::list_mcp_servers(services_path)
        .inspect_err(|e| tracing::warn!(error = %e, "MCP servers unreadable; omitted from the access-control catalogue"))
        .unwrap_or_default()
        .into_iter()
        .map(|s| EntityOption {
            id: s.id.as_str().to_owned(),
            label: s.id.as_str().to_owned(),
            description: s.description,
        })
        .collect()
}

fn build_plugins(services_path: &Path) -> Vec<EntityOption> {
    let admin_roles = vec!["admin".to_owned()];
    repositories::marketplace::plugins::list_plugins_for_roles(services_path, &admin_roles)
        .unwrap_or_default()
        .into_iter()
        .map(|p| EntityOption {
            id: p.id,
            label: p.name,
            description: p.description,
        })
        .collect()
}

fn build_marketplaces() -> Vec<EntityOption> {
    crate::services::marketplaces::load_marketplaces()
        .into_iter()
        .map(|mp| EntityOption {
            id: mp.id.as_str().to_owned(),
            label: mp.name,
            description: mp.description,
        })
        .collect()
}

fn build_agents(services_path: &Path) -> Vec<EntityOption> {
    repositories::config::agents::list_agents(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|a| EntityOption {
            id: a.id.as_str().to_owned(),
            label: a.name,
            description: a.description,
        })
        .collect()
}
