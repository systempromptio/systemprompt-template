//! Server-rendered admin page routes, grouped by dashboard section.

use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router, middleware as axum_middleware};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::templates::AdminTemplateEngine;
use super::super::{handlers, middleware};

pub fn admin_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = super::dashboard_redirects::legacy_routes()
        .merge(super::ssr_dashboard::dashboard_routes())
        .merge(root_routes())
        .merge(access_routes())
        .merge(governance_routes())
        .merge(entity_routes())
        .merge(account_routes())
        .merge(api_routes())
        .merge(super::ssr_redirects::legacy_routes())
        .layer(axum_middleware::from_fn(
            super::ssr_write_gate::require_write_access,
        ))
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
    let public = Router::new()
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
        );
    if handlers::dev_login::dev_login_enabled() {
        public.route(
            "/auth/dev/login",
            get(handlers::dev_login::dev_login_redeem),
        )
    } else {
        public
    }
}

fn root_routes() -> Router<Arc<PgPool>> {
    Router::new().route("/", get(handlers::ssr::overview_page))
}

fn access_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/user", get(handlers::ssr::user_detail_page))
        .route(
            "/departments",
            get(handlers::ssr::management_departments_page),
        )
        .route(
            "/departments/{id}",
            get(handlers::ssr::management_department_detail_page),
        )
        .route(
            "/access-tokens",
            get(handlers::ssr::management_access_tokens_page),
        )
        .route("/tokens/pats", post(handlers::access_tokens::issue_pat))
        .route(
            "/tokens/pats/{id}",
            axum::routing::delete(handlers::access_tokens::revoke_pat),
        )
}

fn governance_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/policies/{policy_id}",
            get(handlers::ssr::governance_policy_edit_page),
        )
        .route(
            "/governance/policies/{policy_id}/toggle",
            post(handlers::ssr::governance_policy_toggle),
        )
        .route(
            "/governance/decisions",
            get(handlers::ssr::governance_decisions_page),
        )
        .route(
            "/governance/hooks",
            get(handlers::ssr::governance_hooks_page),
        )
        .route("/models", get(handlers::ssr::models_page))
        .route("/demo/trace", get(handlers::ssr::demo_trace_page))
}

fn entity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/evals", get(handlers::ssr::evals_page))
        .route("/evals/run", post(handlers::ssr::eval_run_action))
        .route(
            "/evals/cases",
            post(handlers::ssr::eval_promote_case_action),
        )
        .route(
            "/evals/runs/{run_id}",
            get(handlers::ssr::eval_run_detail_page),
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
        .route(
            "/api/profile/salesforce/unlink",
            post(handlers::salesforce_auth::salesforce_unlink),
        )
        .route("/auth/me", get(middleware::auth_me_handler))
        .route(
            "/api/conversations/{session_id}/raw",
            get(handlers::ssr::conversations_raw),
        )
        .route("/api/chain/{id}", get(handlers::ssr::chain_envelope))
        .route("/api/search/resolve", get(handlers::ssr::search_resolve))
}
