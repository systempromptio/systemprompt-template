//! The set of gateway routes this deployment vouches for.
//!
//! Every path that writes a `gateway_route` catalog row — the governance
//! bootstrap, the roles.yaml ingestion it feeds, and the dashboard handlers —
//! derives the set here, from the same `dispatchable_route_ids` the gateway
//! dispatches by, so no two of them can disagree about which ids are real. It
//! includes the synthesized catch-all route, which the services YAML never
//! lists.

use systemprompt::identifiers::RouteId;
use systemprompt::loader::ServicesBootstrap;
use systemprompt::models::ServicesConfig;
use systemprompt::security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_shared::error::MarketplaceError;

pub fn dispatchable_route_ids(services: &ServicesConfig) -> Vec<String> {
    services
        .gateway_config()
        .map(|gateway| gateway.dispatchable_route_ids(&services.providers))
        .unwrap_or_default()
        .iter()
        .map(RouteId::as_str)
        .map(str::to_owned)
        .collect()
}

// Why: an empty set is a services tree without a gateway, not a declaration
// that no route exists — enforcing it would reject every route grant in
// roles.yaml. Such a configuration enforces nothing, and the boot job likewise
// leaves its catalog untouched.
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
        // Why: lint-ok: error-adapt — ConfigLoadError is core's variant-less loader error.
        .map_err(|e| MarketplaceError::Internal(format!("services tree is not loaded: {e}")))?;
    Ok(registered_routes(&dispatchable_route_ids(services)))
}
