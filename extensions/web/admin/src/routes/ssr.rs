//! Server-rendered admin page routes, grouped by dashboard section.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::templates::AdminTemplateEngine;
use super::super::{handlers, middleware};
use super::ssr_redirects;
use crate::handlers::adfs_auth::AdfsDeps;

pub fn admin_ssr_router(
    pool: Arc<PgPool>,
    _write_pool: &PgPool,
    engine: AdminTemplateEngine,
    sso_deps: AdfsDeps,
    _owner: systemprompt::identifiers::UserId,
) -> Router {
    let inner = overview_routes()
        .merge(people_routes())
        .merge(ai_activity_routes())
        .merge(governance_routes())
        .merge(platform_routes())
        .merge(account_routes())
        .merge(api_routes())
        .merge(ssr_redirects::legacy_routes())
        .layer(Extension(engine.clone()))
        .layer(Extension(sso_deps.clone()))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn(
            middleware::non_admin_gate_middleware,
        ))
        .layer(axum_middleware::from_fn(
            middleware::require_user_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::user_context_middleware,
        ))
        .with_state(Arc::clone(&pool));

    let combined = public_routes(Arc::clone(&pool))
        .layer(Extension(engine))
        .layer(Extension(sso_deps))
        .with_state(pool)
        .fallback_service(inner);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(combined),
    )
}

fn public_routes(pool: Arc<PgPool>) -> Router<Arc<PgPool>> {
    let login = Router::new()
        .route("/login", get(handlers::ssr::login_page))
        .layer(axum_middleware::from_fn_with_state(
            pool,
            middleware::user_context_middleware,
        ));
    let routes = login
        .route("/auth/adfs/start", get(handlers::adfs_auth::adfs_start))
        .route("/auth/adfs/acs", post(handlers::adfs_auth::adfs_callback));
    dev_login_routes(routes)
}

// Why: the developer login link is mounted, not merely refused, only on a
// development non-cloud profile — production has no route to hit, so there is
// nothing there to misconfigure open.
fn dev_login_routes(router: Router<Arc<PgPool>>) -> Router<Arc<PgPool>> {
    if !handlers::dev_login::dev_login_enabled() {
        return router;
    }
    router.route(
        "/auth/dev/login",
        get(handlers::dev_login::dev_login_redeem),
    )
}

// Why: Overview is the console landing page, not a redirect — a signed-in
// admin lands on the overview itself, and only a non-console viewer is sent on
// to their profile. The handler makes that call, so `/admin` stays one URL.
fn overview_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/", get(handlers::ssr::overview_page))
}

// Why: sidebar group 2 — the accounts, the groups that carry their
// entitlement, the projects they are attributed to, and the rules that bind
// the three together. Access control moved here from the catalog: it grants
// people access to catalog entries, it is not itself one.
fn people_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/users", get(handlers::ssr::users_page))
        .route(
            "/users/{user_id}",
            get(handlers::ssr::user_detail_by_id_page),
        )
        // Why: the query form is what the roster's row links and the header
        // search resolve to; it stays mounted beside the path form rather
        // than forcing every caller to rewrite its links at once.
        .route("/user", get(handlers::ssr::user_detail_page))
        .route("/groups", get(handlers::ssr::groups_page))
        .route("/groups/{group_id}", get(handlers::ssr::group_detail_page))
        .route("/projects", get(handlers::ssr::projects_page))
        .route(
            "/projects/{project_id}",
            get(handlers::ssr::project_detail_page),
        )
        .route("/roles", get(handlers::ssr::roles_page))
        .route("/devices", get(handlers::ssr::devices_page))
        .route("/access-control", get(handlers::ssr::access_control_page))
        // Why: the token and access-matrix *pages* are gone — entitlement is
        // derived from roles and AD groups, and tokens are minted by the
        // bridge's device-link flow. These endpoints are that flow's API.
        .route("/devices/pats", post(handlers::devices::issue_pat))
        .route(
            "/devices/pats/{id}",
            axum::routing::delete(handlers::devices::revoke_pat),
        )
        .route(
            "/devices/certs/{id}",
            axum::routing::delete(handlers::devices::revoke_cert),
        )
}

// Why: sidebar group 3 — everything that reads what the AI actually did. They
// share one scope contract (`?scope=`/`?range=`), which is why they are one
// group rather than filed under the entity they happen to list.
fn ai_activity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/analytics", get(handlers::ssr::analytics_dashboard_page))
        // Why: the Cost tab's export. Same handler contract as the tab, so the
        // file always matches the view the operator was looking at.
        .route("/analytics/cost.csv", get(handlers::ssr::cost_csv))
        .route("/requests", get(handlers::ssr::analytics_requests_page))
        .route("/requests.csv", get(handlers::ssr::analytics_requests_csv))
        .route(
            "/requests/{request_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/sessions", get(handlers::ssr::sessions_list_page))
        .route(
            "/sessions/{session_id}",
            get(handlers::ssr::session_detail_page),
        )
        .route("/traces", get(handlers::ssr::perf_traces_page))
        .route(
            "/traces/{trace_id}",
            get(handlers::ssr::perf_trace_detail_page),
        )
        // Why: the org-wide twin of "My conversations". It reads one
        // conversation per row where `/contexts` reads one context, and it is
        // the page an operator looks for under AI activity when they want to
        // see what everyone has been asking.
        .route("/conversations", get(handlers::ssr::conversations_page))
        .route("/contexts", get(handlers::ssr::skills_contexts_page))
        .route(
            "/contexts/{context_id}",
            get(handlers::ssr::context_detail_page),
        )
}

// Why: sidebar group 4. These read a posture rather than listing an entity.
// The three are one group because they are the three things a policy can do to
// a call — decide it, hold it for a person, or record a credential it touched —
// and an operator tuning one reads the other two.
fn governance_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/warnings.csv",
            get(handlers::ssr::governance_csv),
        )
        .route(
            "/governance/decisions/{decision_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route("/governance/approvals", get(handlers::ssr::approvals_page))
        .route(
            "/governance/secrets",
            get(handlers::ssr::secrets_audit_page),
        )
        .route(
            "/governance/secrets.csv",
            get(handlers::ssr::secrets_audit_csv),
        )
}

// Why: sidebar group 5 — the installable units declared in `services/*.yaml`,
// flattened out of the old `/catalog/` prefix, plus the gateway that routes
// model traffic to the providers behind them.
fn platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/mcp", get(handlers::catalog::mcp::mcp_servers_page))
        .route(
            "/mcp/{mcp_id}",
            get(handlers::catalog::mcp::mcp_detail_page),
        )
        .route(
            "/marketplaces",
            get(handlers::catalog::marketplaces::marketplaces_page),
        )
        .route(
            "/marketplaces/{marketplace_id}",
            get(handlers::catalog::marketplaces::marketplace_detail_page),
        )
        .route("/plugins", get(handlers::catalog::plugins_page))
        .route(
            "/plugins/{plugin_id}",
            get(handlers::catalog::plugin_detail_page),
        )
        .route("/skills", get(handlers::catalog::skills_page))
        .route(
            "/skills/{skill_id}",
            get(handlers::catalog::skill_detail_page),
        )
        .route("/gateway", get(handlers::ssr::gateway_page))
        // Why: the month-end pack's *pages* are gone — the cost tab of the
        // analytics dashboard replaced them — but the CSV exports are a data
        // endpoint the finance hand-off still fetches, so they stay mounted.
        .route(
            "/reports/customer.csv",
            get(handlers::ssr::report_customer_csv),
        )
        .route(
            "/reports/internal.csv",
            get(handlers::ssr::report_internal_csv),
        )
}

fn account_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/profile", get(handlers::ssr::profile_page))
        // Why: identity-scoped, not admin-gated — the handler resolves what
        // the viewer may see (self, or everything for admins) per request.
        .route("/history", get(handlers::ssr::history_page))
        // Why: owner-facing, so it sits in `account_routes` beside `/history`
        // rather than under the admin-gated `/entities/` tree. The handler
        // resolves ownership per request and 404s a viewer who is not the owner.
        .route(
            "/history/conversations/{context_id}",
            get(handlers::ssr::history_conversation_page),
        )
        .route("/settings", get(handlers::ssr::settings_page))
        .route("/setup", get(handlers::ssr::setup_page))
}

fn api_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/auth/me", get(middleware::auth_me_handler))
        .route(
            "/api/conversations/{session_id}/raw",
            get(handlers::ssr::conversations_raw),
        )
        .route("/api/chain/{id}", get(handlers::ssr::chain_envelope))
        .route("/api/history/search", get(handlers::ssr::history_search))
        .route("/api/search/resolve", get(handlers::ssr::search_resolve))
        .route(
            "/api/profile/bridge-code",
            post(handlers::ssr::issue_bridge_code),
        )
        .route(
            "/api/profile/salesforce/unlink",
            post(handlers::salesforce_auth::salesforce_unlink),
        )
}
