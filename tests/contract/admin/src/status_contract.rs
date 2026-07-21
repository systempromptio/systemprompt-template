//! The contract itself: drive every mounted route under every principal and
//! assert the properties that must hold regardless of which status a given
//! route happens to return.
//!
//! One test, not one per route, because the globals in [`crate::globals`] are
//! process-wide `OnceLock`s and the throwaway database is expensive to build;
//! a single pass keeps both to one setup and reports every violation at once
//! rather than the first.

use axum::http::StatusCode;

use crate::app::App;
use crate::principal::Principal;
use crate::route_source::{MountedRoute, mounted_routes};
use crate::{baseline, globals, principal, tempdb::TempDb};

// Routes known to answer 5xx today, each with the defect that causes it.
//
// This list exists so the no-5xx invariant can be enforced for everything
// else while the error model is still being adopted. It is checked in both
// directions — an entry whose route stops failing must be deleted — so it can
// only ever shrink.
const KNOWN_5XX: [(&str, &str); 0] = [];

fn known_5xx(key: &str) -> bool {
    KNOWN_5XX.iter().any(|(route, _)| *route == key)
}

// Routes served before authentication, by design. Everything else must
// refuse an anonymous caller.
fn is_public(template: &str) -> bool {
    const PUBLIC: [&str; 7] = [
        "/admin/login",
        "/admin/register",
        "/admin/add-passkey",
        "/admin/verify-pending",
        "/admin/api/magic-link/request",
        "/admin/api/magic-link/validate",
        "/admin/api/register",
    ];
    PUBLIC.contains(&template)
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_routes_honour_their_http_contract() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping admin HTTP contract suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let routes = mounted_routes();
    assert!(
        routes.len() >= 80,
        "only {} routes discovered — the route-source parser is missing declarations",
        routes.len()
    );

    let mut observed = Vec::with_capacity(routes.len());
    let mut violations = Vec::new();
    let mut still_failing: Vec<String> = Vec::new();

    for route in routes {
        let mut statuses = Vec::with_capacity(Principal::ALL.len());
        for principal in Principal::ALL {
            let (status, body) = app
                .send(&route.method, &route.request_path(), principal)
                .await;
            check(&route, principal, status, body.as_deref(), &mut violations);
            if status.is_server_error() {
                still_failing.push(route.key());
            }
            statuses.push((principal, status));
        }
        observed.push((route, statuses));
    }

    db.cleanup().await;

    assert!(
        violations.is_empty(),
        "{} route(s) violate the HTTP contract:\n{}",
        violations.len(),
        violations.join("\n")
    );

    let stale: Vec<&str> = KNOWN_5XX
        .iter()
        .map(|(route, _)| *route)
        .filter(|route| !still_failing.iter().any(|k| k == route))
        .collect();
    assert!(
        stale.is_empty(),
        "KNOWN_5XX lists {} route(s) that no longer return 5xx — delete them so the list keeps \
         reflecting real defects:\n  {}",
        stale.len(),
        stale.join("\n  ")
    );

    baseline::assert_matches(&baseline::render(&observed));
}

// The invariants that hold for every route, whatever its baseline status.
fn check(
    route: &MountedRoute,
    principal: Principal,
    status: StatusCode,
    body: Option<&str>,
    violations: &mut Vec<String>,
) {
    let mut fail = |why: &str| {
        let detail = body.map_or_else(String::new, |b| format!("\n      body: {b}"));
        violations.push(format!(
            "  {} [{}] -> {} : {why}{detail}",
            route.key(),
            principal.label(),
            status.as_u16()
        ));
    };

    // A well-formed request must never produce a server error. This is the
    // property Category E must not regress, and the one that catches an
    // `unwrap_or_default()` rendering a page over a failed query.
    if status.is_server_error() && !known_5xx(&route.key()) {
        fail("well-formed requests must never produce a 5xx");
    }

    // An anonymous caller must be refused or redirected — never served.
    if principal == Principal::Anonymous
        && !is_public(&route.template)
        && status.is_success()
    {
        fail("anonymous callers must not be served a success response");
    }

    // An authenticated admin must never be turned away by the auth layers.
    if principal == Principal::Admin
        && matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
    {
        fail("an admin must not be rejected by authentication or authorisation");
    }
}
