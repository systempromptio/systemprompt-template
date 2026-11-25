//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the rules.yaml projection it feeds, and the dashboard handlers
//! — derives the set here, from the resolved catalog's declared routes, the
//! same routes the gateway dispatches by, so no two of them can disagree
//! about which ids are real. The catalog is read from the install services
//! view the web member resolved.

use crate::env::AdminEnv;

use systemprompt::governance::authz::{EntityKind, RegisteredEntities};
use systemprompt::manifest::services::{GatewayRoute, ResolvedCatalog};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::{GatewayRouteView, RouteFallbackView};

#[must_use]
pub fn dispatchable_route_ids(catalog: &ResolvedCatalog) -> Vec<String> {
    catalog
        .routes()
        .iter()
        .map(|route| route.id.as_str().to_owned())
        .collect()
}

// Why: a read-only projection for the access-control views: the route's
// model globs, its first deployment's provider, and the remaining deployments
// as the fallback chain. Nothing here is written back.
fn route_view(route: &GatewayRoute) -> GatewayRouteView {
    let mut deployments = route
        .deployments
        .iter()
        .map(|d| d.endpoint().provider.as_str());
    GatewayRouteView {
        id: route.id.as_str().to_owned(),
        name: route.label.clone(),
        description: route.description.clone(),
        model_pattern: route.models.0.join(" "),
        provider: deployments.next().unwrap_or_default().to_owned(),
        upstream_model: None,
        extra_headers: std::collections::BTreeMap::new(),
        pricing: None,
        when: None,
        requires: None,
        fallbacks: deployments
            .map(|provider| RouteFallbackView {
                provider: provider.to_owned(),
                upstream_model: None,
            })
            .collect(),
        by_scope: None,
    }
}

pub fn dispatchable_routes(
    catalog: &ResolvedCatalog,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    if catalog.routes().is_empty() {
        return Err(missing_gateway());
    }
    Ok(catalog.routes().iter().map(route_view).collect())
}

// Why: an absent catalog is an error, never an empty list. The per-user
// catalog and the after-the-fact ACL detector both iterate these routes, so
// returning nothing would let them report no violations while checking
// nothing — a governance surface may not fail open and quiet.
fn missing_gateway() -> MarketplaceError {
    MarketplaceError::Internal(
        "no gateway routes in the services catalog — expected a `gateway.routes` block in \
         services/ai/gateway.yaml"
            .to_owned(),
    )
}

pub fn dispatchable_routes_from_services(
    env: &AdminEnv,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    dispatchable_routes(env.services_view().catalog())
}

// Why: the routes a client may name directly: a route matching at least one
// model the catalog advertises to some client family. A route that reaches
// only unadvertised models serves an `upstream_model` rewrite, never a name a
// client targets, so listing it in a per-user catalog advertises egress the
// gateway would refuse.
pub fn client_facing_routes(
    catalog: &ResolvedCatalog,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    if catalog.routes().is_empty() {
        return Err(missing_gateway());
    }
    Ok(catalog
        .routes()
        .iter()
        .filter(|route| {
            catalog
                .models()
                .any(|model| model.is_advertised() && route.matches_model(model.id.as_str()))
        })
        .map(route_view)
        .collect())
}

pub fn client_facing_routes_from_services(
    env: &AdminEnv,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    client_facing_routes(env.services_view().catalog())
}

// Why: an empty set is a services tree without a gateway, not a declaration
// that no route exists â enforcing it would reject every route grant in
// rules.yaml. Such a tree enforces nothing, and the boot job likewise leaves
// its catalog untouched.
#[must_use]
pub fn registered_routes(route_ids: &[String]) -> RegisteredEntities {
    if route_ids.is_empty() {
        RegisteredEntities::default()
    } else {
        RegisteredEntities::new().with_kind(EntityKind::GatewayRoute, route_ids.iter().cloned())
    }
}

#[must_use]
pub fn registered_routes_from_services(env: &AdminEnv) -> RegisteredEntities {
    registered_routes(&dispatchable_route_ids(env.services_view().catalog()))
}
