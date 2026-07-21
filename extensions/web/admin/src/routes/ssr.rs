//! Server-rendered admin page routes, grouped by dashboard section.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::templates::AdminTemplateEngine;
use super::super::{handlers, middleware};

pub fn admin_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = root_routes()
        .merge(access_routes())
        .merge(catalog_routes())
        .merge(governance_routes())
        .merge(entity_routes())
        .merge(account_routes())
        .merge(api_routes())
        .layer(Extension(engine.clone()))
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
        .route("/register", get(handlers::ssr::register_page))
        .route("/add-passkey", get(handlers::ssr::add_passkey_page))
        .route("/verify-pending", get(handlers::ssr::verify_pending_page))
        .route(
            "/api/magic-link/request",
            post(handlers::magic_link::request_magic_link),
        )
        .route(
            "/api/magic-link/validate",
            post(handlers::magic_link::validate_magic_link),
        )
        .route(
            "/api/register",
            post(handlers::public_register::public_register_handler),
        )
}

fn root_routes() -> Router<Arc<PgPool>> {
    Router::new().route(
        "/",
        get(|| async { axum::response::Redirect::to("/admin/profile") }),
    )
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
        .route(
            "/access/devices",
            get(handlers::ssr::management_devices_page),
        )
        .route("/access/matrix", get(handlers::ssr::access_control_page))
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
            get(|| async { axum::response::Redirect::permanent("/admin/catalog/marketplace") }),
        )
        .route(
            "/catalog/marketplace",
            get(handlers::catalog::marketplace_page),
        )
        .route("/catalog/a2a", get(handlers::catalog::a2a_agents_page))
        .route(
            "/catalog/external",
            get(handlers::catalog::external_agents_page),
        )
}

fn governance_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance/policies", get(handlers::ssr::governance_page))
        .route(
            "/governance/policies/{policy_id}",
            get(handlers::ssr::governance_policy_edit_page),
        )
        .route(
            "/governance/policies/{policy_id}/toggle",
            post(handlers::ssr::governance_policy_toggle),
        )
        .route(
            "/governance/hooks",
            get(handlers::ssr::governance_hooks_page),
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
        .route("/demo-register", get(handlers::ssr::demo_register_page))
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
}
