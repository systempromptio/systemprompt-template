//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the roles.yaml ingestion it feeds, and the dashboard handlers —
//! derives the set here, from the same `dispatchable_route_ids` the gateway
//! dispatches by, so no two of them can disagree about which ids are real. It
//! includes the synthesized catch-all route, which the gateway YAML never
//! lists. The routes and the registry they resolve against are services
//! configuration (`services/ai/gateway.yaml`, `services/ai/providers.yaml`),
//! read from the `ServicesBootstrap` cell the process booted with.

use systemprompt::identifiers::RouteId;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::models::services::ProviderRegistry;
use systemprompt::security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::GatewayRouteView;

pub fn dispatchable_route_ids(services: &ServicesConfig) -> Vec<String> {
    services
        .gateway
        .as_ref()
        .map(|gateway| gateway.dispatchable_route_ids(&services.providers))
        .unwrap_or_default()
        .iter()
        .map(RouteId::as_str)
        .map(str::to_owned)
        .collect()
}

// Why: the same routes as `dispatchable_route_ids`, carrying the metadata the
// access-control views label and filter by. Read views that decide *access*
// must use this rather than the gateway YAML `get_gateway_config` reads: that
// YAML omits the synthesized catch-all, so a grant on it would be invisible —
// and therefore unreachable — from any surface built on the file.
pub fn dispatchable_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    let config = services.gateway_config().ok_or_else(missing_gateway)?;
    Ok(config
        .candidate_routes(&services.providers)
        .map(|route| {
            let route = route.into_owned();
            GatewayRouteView {
                id: route.id.as_str().to_owned(),
                model_pattern: route.model_pattern,
                provider: route.provider.as_str().to_owned(),
                upstream_model: route.upstream_model,
                extra_headers: route.extra_headers.into_iter().collect(),
                // Why: read-only projection for the access-control views —
                // nothing here is ever written back to the file, so the
                // editor's passthrough fields are dropped rather than
                // round-tripped through a second representation.
                pricing: None,
                when: None,
                requires: None,
            }
        })
        .collect())
}

// Why: an absent catalog is an error, never an empty list. The per-user
// catalog and the after-the-fact ACL detector both iterate these routes, so
// returning nothing would let them report no violations while checking
// nothing — a governance surface may not fail open and quiet.
fn missing_gateway() -> MarketplaceError {
    MarketplaceError::Internal(
        "no gateway configuration in the services tree — expected a `gateway:` block in \
         services/ai/gateway.yaml, included from services/config/config.yaml"
            .to_owned(),
    )
}

pub fn dispatchable_routes_from_services() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    dispatchable_routes(
        ServicesBootstrap::get()
            .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?,
    )
}

// Why: the routes a client may name directly. A provider declared
// `surface: backend` in `services/ai/providers.yaml` is a dispatch target for
// an `upstream_model` rewrite, never a surface a client targets, so listing it
// in a per-user catalog advertises egress the gateway would refuse. Same rule
// `/v1/models` applies via `ApiSurface::is_advertised`. A route naming a
// provider the registry does not know passes through: only a provider that
// declares itself backend is hidden, and an unknown one is a validation
// problem for the services loader, not a reason for the catalog to go quiet.
pub fn client_facing_routes(
    services: &ServicesConfig,
) -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    Ok(retain_client_facing(
        dispatchable_routes(services)?,
        &services.providers,
    ))
}

#[must_use]
pub fn retain_client_facing(
    routes: Vec<GatewayRouteView>,
    providers: &ProviderRegistry,
) -> Vec<GatewayRouteView> {
    routes
        .into_iter()
        .filter(|route| {
            providers
                .find_provider(&route.provider)
                .is_none_or(|entry| entry.surface.is_advertised())
        })
        .collect()
}

pub fn client_facing_routes_from_services() -> Result<Vec<GatewayRouteView>, MarketplaceError> {
    client_facing_routes(
        ServicesBootstrap::get()
            .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?,
    )
}

// Why: an empty set is a services tree without a gateway, not a declaration
// that no route exists — enforcing it would reject every route grant in
// roles.yaml. Such a tree enforces nothing, and the boot job likewise leaves
// its catalog untouched.
#[must_use]
pub fn registered_routes(route_ids: &[String]) -> RegisteredEntities {
    if route_ids.is_empty() {
        RegisteredEntities::default()
    } else {
        RegisteredEntities::new().with_kind(EntityKind::GatewayRoute, route_ids.iter().cloned())
    }
}

pub fn registered_routes_from_services() -> Result<RegisteredEntities, MarketplaceError> {
    let services = ServicesBootstrap::get()
        .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?;
    Ok(registered_routes(&dispatchable_route_ids(services)))
}
