#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]
//! The gateway entity catalog: which route ids this deployment vouches for,
//! and what happens to a grant naming one it does not.
//!
//! The load-bearing case is the empty set. Core reads a declared-but-empty
//! kind as "this deployment has none of these" and rejects every id of it, so
//! a profile without a gateway would reject every route grant in roles.yaml.
//! `registered_routes` has to turn that into "enforce nothing" instead.

use axum::http::StatusCode;
use systemprompt::security::authz::{AuthzError, EntityKind};
use systemprompt_web_admin::error::AdminError;
use systemprompt_web_admin::repositories::config::gateway::registered_routes;

fn ids(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn a_declared_route_set_admits_its_own_ids_and_refuses_the_rest() {
    let registered = registered_routes(&ids(&["route-abc123", "route-def456"]));

    registered
        .require(EntityKind::GatewayRoute, "route-abc123")
        .expect("a declared id is admitted");
    let err = registered
        .require(EntityKind::GatewayRoute, "route-typo")
        .expect_err("an id no route claims is refused");
    assert!(
        matches!(err, AuthzError::Validation(_)),
        "an unregistered id is the caller's mistake, not an internal failure: {err:?}"
    );
}

#[test]
fn an_empty_route_set_enforces_nothing_rather_than_refusing_everything() {
    let registered = registered_routes(&[]);

    registered
        .require(EntityKind::GatewayRoute, "route-abc123")
        .expect("a gateway-less profile makes no claim about route ids");
}

#[test]
fn only_the_gateway_route_kind_is_enforced() {
    let registered = registered_routes(&ids(&["route-abc123"]));

    registered
        .require(EntityKind::McpServer, "knowledge-bank")
        .expect("kinds this deployment does not declare keep self-materialising");
}

#[test]
fn an_unregistered_id_answers_400_not_500() {
    let err = registered_routes(&ids(&["route-abc123"]))
        .require(EntityKind::GatewayRoute, "route-typo")
        .expect_err("refused");

    assert_eq!(AdminError::from(err).status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        AdminError::from(AuthzError::Validation("bad id".to_owned())).status(),
        StatusCode::BAD_REQUEST
    );
}
