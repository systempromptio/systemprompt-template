//! The declaration: the `gateway.routes:` sequence of
//! `services/ai/gateway.yaml`, in file order.
//!
//! Read from the baked services tree — the file the editor writes and core
//! boots from — and hashed on the parsed routes so a comment or whitespace
//! edit is not a drift. Every field core's `GatewayRoute` carries is part of
//! the hash, the fallback pair included.

use std::path::Path;

use serde_yaml::Value;
use sha2::{Digest, Sha256};

use crate::error::{AdminError, AdminResult};
use crate::repositories::config::gateway::route_from_yaml;
use crate::types::GatewayRouteView;

pub const GATEWAY_FILE: &str = "ai/gateway.yaml";

// Why: one declared route and whether the file spelled its `id:` out.
#[derive(Debug, Clone)]
pub struct DeclaredRoute {
    pub route: GatewayRouteView,
    pub explicit_id: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DeclaredRoutes {
    pub routes: Vec<DeclaredRoute>,
}

// Why: the canonical line for one route — the same function keys the hash
// and the drift comparison, so the two cannot disagree about what counts.
#[must_use]
pub fn route_fingerprint(route: &GatewayRouteView) -> String {
    let block = |v: Option<&Value>| {
        // Why: discard-ok: a block that parsed as YAML serialises; an empty
        // string only makes the fingerprint differ
        v.map(|v| serde_yaml::to_string(v).unwrap_or_default())
            .unwrap_or_default()
    };
    let headers = route
        .extra_headers
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        route.id,
        route.name.as_deref().unwrap_or_default(),
        route.description.as_deref().unwrap_or_default(),
        route.model_pattern,
        route.provider,
        route.upstream_model.as_deref().unwrap_or_default(),
        headers,
        block(route.pricing.as_ref()),
        block(route.when.as_ref()),
        block(route.requires.as_ref()),
        route.fallback_provider.as_deref().unwrap_or_default(),
        route.fallback_upstream_model.as_deref().unwrap_or_default(),
    )
}

impl DeclaredRoutes {
    #[must_use]
    pub fn declared_hash(&self) -> String {
        let mut hasher = Sha256::new();
        for (i, d) in self.routes.iter().enumerate() {
            hasher.update(format!("route\t{i}\t{}\n", route_fingerprint(&d.route)).as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    #[must_use]
    pub fn find(&self, id: &str) -> Option<&DeclaredRoute> {
        self.routes.iter().find(|d| d.route.id == id)
    }
}

pub fn parse_declared_routes(yaml: &str) -> Result<DeclaredRoutes, String> {
    let doc: Value = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
    let Some(seq) = doc
        .get("gateway")
        .and_then(|g| g.get("routes"))
        .and_then(Value::as_sequence)
    else {
        return Ok(DeclaredRoutes::default());
    };
    let mut routes = Vec::with_capacity(seq.len());
    for (i, entry) in seq.iter().enumerate() {
        let Some(route) = route_from_yaml(entry) else {
            return Err(format!(
                "gateway.routes[{i}] lacks model_pattern or provider"
            ));
        };
        let explicit_id = entry
            .as_mapping()
            .is_some_and(|m| m.contains_key(Value::from("id")));
        if routes
            .iter()
            .any(|d: &DeclaredRoute| d.route.id == route.id)
        {
            return Err(format!("gateway.routes[{i}] repeats id `{}`", route.id));
        }
        routes.push(DeclaredRoute { route, explicit_id });
    }
    Ok(DeclaredRoutes { routes })
}

pub fn load_declared_routes(services_path: &Path) -> AdminResult<DeclaredRoutes> {
    let path = services_path.join(GATEWAY_FILE);
    let yaml = std::fs::read_to_string(&path)
        .map_err(|e| AdminError::invalid("gateway.yaml could not be read", e))?;
    parse_declared_routes(&yaml)
        .map_err(|e| AdminError::invalid("gateway.yaml could not be parsed", e))
}

// Why: the baked tree, not the composed root — the composed root is a
// content-addressed overlay the next refresh replaces, and this is the one
// file the console writes back, so declaration and runtime must be the
// same path.
pub fn baked_services_path() -> AdminResult<std::path::PathBuf> {
    Ok(std::path::PathBuf::from(
        &systemprompt::config::ProfileBootstrap::get()?
            .paths
            .services,
    ))
}

pub fn gateway_file_path() -> AdminResult<std::path::PathBuf> {
    Ok(baked_services_path()?.join(GATEWAY_FILE))
}

pub fn declared_routes_now() -> AdminResult<DeclaredRoutes> {
    load_declared_routes(&baked_services_path()?)
}
