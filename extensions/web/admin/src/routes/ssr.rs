//! Server-rendered admin page routes, grouped by dashboard section.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::templates::AdminTemplateEngine;
use super::super::{handlers, middleware};
use crate::handlers::salesforce_auth::SalesforceDeps;

pub fn admin_ssr_router(
    pool: Arc<PgPool>,
    engine: AdminTemplateEngine,
    sf_deps: SalesforceDeps,
) -> Router {
    let inner = root_routes()
        .merge(analytics_routes())
        .merge(enterprise_routes())
        .merge(access_routes())
        .merge(catalog_routes())
        .merge(entity_routes())
        .merge(account_routes())
        .merge(api_routes())
        .layer(Extension(engine.clone()))
        .layer(Extension(sf_deps.clone()))
        .layer(axum_middleware::from_fn(
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

    let combined = public_routes()
        .layer(Extension(engine))
        .layer(Extension(sf_deps))
        .with_state(pool)
        .fallback_service(inner);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(combined),
    )
}

fn public_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/login", get(handlers::ssr::login_page))
        .route(
            "/auth/salesforce/start",
            get(handlers::salesforce_auth::salesforce_start),
        )
        .route(
            "/auth/salesforce/callback",
            get(handlers::salesforce_auth::salesforce_callback),
        )
        .route(
            "/auth/passkey/register",
            post(handlers::passkey_auth::passkey_register),
        )
        // Why: public by design — the invite token in the URL/body is the
        // authorization, exactly like the passkey register door above.
        .route("/invite/{token}", get(handlers::ssr::invite_accept_page))
        .route(
            "/auth/invite/accept",
            post(handlers::invites::accept_invite_handler),
        )
}

fn root_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/", get(root_redirect))
}

// Why: admins land on the analytics dashboard; everyone else still gets their
// profile — the non-admin gate would bounce them off /admin/analytics anyway.
async fn root_redirect(
    Extension(user_ctx): Extension<crate::types::UserContext>,
) -> axum::response::Redirect {
    if user_ctx.is_admin {
        axum::response::Redirect::to("/admin/analytics")
    } else {
        axum::response::Redirect::to("/admin/profile")
    }
}

// Why: admin-scoped rather than platform-scoped — an organization's own
// administrator is a first-class viewer of their own data; the handler locks
// their scope to their organization.
fn analytics_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/analytics", get(handlers::ssr::analytics_dashboard_page))
}

fn enterprise_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/enterprises", get(handlers::ssr::enterprises_page))
        .route(
            "/enterprises/{slug}",
            get(handlers::ssr::enterprise_detail_page),
        )
        .route(
            "/reports/internal",
            get(handlers::ssr::report_internal_page),
        )
        .layer(axum_middleware::from_fn(
            middleware::require_platform_admin_middleware,
        ))
}

fn access_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/access/users", get(handlers::ssr::users_page))
        .route("/access/user", get(handlers::ssr::user_detail_page))
        .route("/user", get(handlers::ssr::user_detail_page))
        .route(
            "/access/departments",
            get(handlers::ssr::management_departments_page),
        )
        .route(
            "/access/departments/{id}",
            get(handlers::ssr::management_department_detail_page),
        )
        // Why: the customer report is admin-scoped rather than
        // platform-scoped — a customer's own administrator may read their own
        // organization's usage, and the handler is what decides whose.
        .route(
            "/reports/customer",
            get(handlers::ssr::report_customer_page),
        )
        // Why: the token and access-matrix *pages* are gone — entitlement is
        // derived from the organization's plan, and tokens are minted by the
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

fn catalog_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/catalog",
            get(|| async { axum::response::Redirect::permanent("/admin/catalog/plugins") }),
        )
        .route(
            "/catalog/marketplace",
            get(|| async { axum::response::Redirect::permanent("/admin/catalog/plugins") }),
        )
        .route("/catalog/plugins", get(handlers::catalog::plugins_page))
        .route(
            "/catalog/plugins/{plugin_id}",
            get(handlers::catalog::plugin_detail_page),
        )
        .route("/catalog/skills", get(handlers::catalog::skills_page))
        .route(
            "/catalog/skills/{skill_id}",
            get(handlers::catalog::skill_detail_page),
        )
        .route("/catalog/mcp", get(handlers::catalog::mcp_servers_page))
        .route(
            "/catalog/mcp/{mcp_id}",
            get(handlers::catalog::mcp_detail_page),
        )
}

fn entity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/entities/requests",
            get(handlers::ssr::analytics_requests_page),
        )
        .route(
            "/entities/requests/{request_id}",
            get(handlers::ssr::governance_audit_detail_page),
        )
        .route(
            "/entities/sessions",
            get(handlers::ssr::users_sessions_page),
        )
        .route(
            "/entities/sessions/{session_id}",
            get(handlers::ssr::session_detail_page),
        )
        .route("/entities/traces", get(handlers::ssr::perf_traces_page))
        .route(
            "/entities/traces/{trace_id}",
            get(handlers::ssr::perf_trace_detail_page),
        )
        .route(
            "/entities/contexts",
            get(handlers::ssr::skills_contexts_page),
        )
        .route(
            "/entities/contexts/{context_id}",
            get(handlers::ssr::context_detail_page),
        )
}

fn account_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/profile", get(handlers::ssr::profile_page))
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
        .route("/api/search/resolve", get(handlers::ssr::search_resolve))
        .route(
            "/api/profile/salesforce/unlink",
            post(handlers::salesforce_auth::salesforce_unlink),
        )
}
