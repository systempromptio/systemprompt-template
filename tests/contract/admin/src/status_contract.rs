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
use crate::{baseline, globals, principal};
use template_test_common::TempDb;

// Routes known to answer 5xx today, each with the defect that causes it.
//
// This list exists so the no-5xx invariant can be enforced for everything
// else while the error model is still being adopted. It is checked in both
// directions — an entry whose route stops failing must be deleted — so it can
// only ever shrink.
const KNOWN_5XX: [(&str, &str); 1] = [(
    "POST /api/public/admin/sync/sources/refresh",
    "runs core's in-process ServicesRefresh, which core layers onto extension routers at \
     mount time; the contract harness mounts the admin router without it, so the missing \
     extension is the only answer it can see",
)];

fn known_5xx(key: &str) -> bool {
    KNOWN_5XX.iter().any(|(route, _)| *route == key)
}

// Routes on the platform tier: an AD mapping decides what the directory grants
// everyone in that AD group, so writing one is above an ordinary admin
// (`admin_groups::group_platform_routes` / `project_platform_routes`). For
// these the no-rejection invariant is held by the platform admin, and an
// ordinary admin must be refused.
fn is_platform_tier(key: &str) -> bool {
    const PLATFORM: [&str; 4] = [
        "POST /api/public/admin/groups/{group_id}/ad-mappings",
        "DELETE /api/public/admin/groups/{group_id}/ad-mappings/{ad_group}",
        "POST /api/public/admin/projects/{project_id}/ad-mappings",
        "DELETE /api/public/admin/projects/{project_id}/ad-mappings/{ad_group}",
    ];
    PLATFORM.contains(&key)
}

// Routes served before authentication, by design. Everything else must
// refuse an anonymous caller.
fn is_public(template: &str) -> bool {
    template == "/admin/login"
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

    let credentials = principal::provision_dashboard(&db.pool).await;
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
    if principal == Principal::Anonymous && !is_public(&route.template) && status.is_success() {
        fail("anonymous callers must not be served a success response");
    }

    // An authenticated admin must never be turned away by the auth layers,
    // except on the platform tier, where the platform admin holds that place
    // and an ordinary admin must be refused.
    let rejected = matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN);
    let platform = is_platform_tier(&route.key());
    if principal == Principal::Admin && rejected && !platform {
        fail("an admin must not be rejected by authentication or authorisation");
    }
    if principal == Principal::Admin && platform && status != StatusCode::FORBIDDEN {
        fail("an ordinary admin must be refused a platform-tier write");
    }
    if principal == Principal::PlatformAdmin && rejected && platform {
        fail("a platform admin must not be rejected on the platform tier");
    }
}
