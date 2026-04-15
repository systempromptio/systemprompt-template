use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::super::{
    event_hub, handlers, middleware, templates::AdminTemplateEngine, tier_enforcement,
};

pub fn workspace_ssr_router(
    pool: Arc<PgPool>,
    engine: AdminTemplateEngine,
    hub: event_hub::EventHub,
    ai_service: Option<Arc<systemprompt::ai::AiService>>,
    tier_cache: tier_enforcement::TierEnforcementCache,
) -> Router {
    let inner = workspace_routes()
        .layer(Extension(hub))
        .layer(Extension(ai_service))
        .layer(Extension(tier_cache))
        .layer(Extension(engine))
        .layer(axum_middleware::from_fn(
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn(
            middleware::require_user_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            Arc::clone(&pool),
            middleware::user_context_middleware,
        ))
        .with_state(pool);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(inner),
    )
}

fn workspace_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/", get(handlers::ssr::control_center_page))
        .route("/api/sse", get(handlers::sse::control_center_sse))
        .route(
            "/api/rate-session",
            post(handlers::ssr::handle_rate_session),
        )
        .route("/api/rate-skill", post(handlers::ssr::handle_rate_skill))
        .route(
            "/api/session-status",
            post(handlers::ssr::handle_update_session_status),
        )
        .route(
            "/api/batch-session-status",
            post(handlers::ssr::handle_batch_update_session_status),
        )
        .route(
            "/api/analyse-session",
            post(handlers::ssr::handle_analyse_session),
        )
        .route(
            "/api/generate-report",
            post(handlers::ssr::handle_generate_report),
        )
        .route(
            "/api/generate-profile-report",
            post(handlers::ssr::handle_generate_profile_report),
        )
}

pub fn admin_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = dashboard_routes()
        .merge(user_page_routes())
        .merge(my_routes())
        .merge(entity_routes())
        .merge(org_routes())
        .route("/setup", get(handlers::ssr::setup_page))
        .route("/demo-register", get(handlers::ssr::demo_register_page))
        .route("/auth/me", get(middleware::auth_me_handler))
        .layer(Extension(engine.clone()))
        .layer(axum_middleware::from_fn(
            middleware::marketplace_context_middleware,
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

fn dashboard_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/",
            get(|| async { axum::response::Redirect::to("/admin/dashboard") }),
        )
        .route("/dashboard", get(handlers::ssr::dashboard_page))
        .route(
            "/api/generate-traffic-report",
            post(handlers::ssr::handle_generate_traffic_report),
        )
        .route("/api/sse/dashboard", get(handlers::sse::dashboard_sse))
}

fn user_page_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/users", get(handlers::ssr::users_page))
        .route("/jobs", get(handlers::ssr::jobs_page))
        .route("/events", get(handlers::ssr::events_page))
        .route("/profile", get(handlers::ssr::profile_page))
        .route("/settings", get(handlers::ssr::settings_page))
        .route("/achievements", get(handlers::ssr::achievements_page))
        .route("/leaderboard", get(handlers::ssr::leaderboard_page))
        .route("/user", get(handlers::ssr::user_detail_page))
        .route("/governance", get(handlers::ssr::governance_page))
        .route(
            "/governance/decisions",
            get(handlers::ssr::governance_decisions_page),
        )
        .route("/governance/audit", get(handlers::ssr::governance_audit_page))
        .route("/governance/rules", get(handlers::ssr::governance_rules_page))
        .route(
            "/governance/violations",
            get(handlers::ssr::governance_violations_page),
        )
        .route(
            "/governance/rate-limits",
            get(handlers::ssr::governance_rate_limits_page),
        )
        .route("/governance/hooks", get(handlers::ssr::governance_hooks_page))
        .route("/traces", get(handlers::ssr::traces_page))
        .route("/access-control", get(handlers::ssr::access_control_page))
        .route("/analytics", get(handlers::ssr::analytics_overview_page))
        .route(
            "/analytics/agents",
            get(handlers::ssr::analytics_agents_page),
        )
        .route("/analytics/costs", get(handlers::ssr::analytics_costs_page))
        .route(
            "/analytics/requests",
            get(handlers::ssr::analytics_requests_page),
        )
        .route(
            "/analytics/sessions",
            get(handlers::ssr::analytics_sessions_page),
        )
        .route(
            "/analytics/content",
            get(handlers::ssr::analytics_content_page),
        )
        .route(
            "/analytics/conversations",
            get(handlers::ssr::analytics_conversations_page),
        )
        .route("/analytics/tools", get(handlers::ssr::analytics_tools_page))
        .route(
            "/infrastructure/services",
            get(handlers::ssr::infra_services_page),
        )
        .route(
            "/infrastructure/database",
            get(handlers::ssr::infra_database_page),
        )
        .route("/infrastructure/logs", get(handlers::ssr::infra_logs_page))
        .route(
            "/infrastructure/config",
            get(handlers::ssr::infra_config_page),
        )
        .route("/mcp/access", get(handlers::ssr::mcp_access_page))
        .route("/mcp/tools", get(handlers::ssr::mcp_tools_page))
        .route("/performance/traces", get(handlers::ssr::perf_traces_page))
        .route(
            "/performance/benchmarks",
            get(handlers::ssr::perf_benchmarks_page),
        )
        .route("/users/sessions", get(handlers::ssr::users_sessions_page))
        .route("/users/ip-bans", get(handlers::ssr::users_ip_bans_page))
}

fn my_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/my/marketplace", get(handlers::ssr::my_marketplace_page))
        .route("/my/plugins", get(handlers::ssr::my_plugins_page))
        .route("/browse/plugins", get(handlers::ssr::browse_plugins_page))
        .route("/my/plugins/view", get(handlers::ssr::my_plugin_view_page))
        .route("/my/plugins/edit", get(handlers::ssr::my_plugin_edit_page))
        .route("/my/skills", get(handlers::ssr::my_skills_page))
        .route("/my/skills/edit", get(handlers::ssr::my_skill_edit_page))
        .route("/my/agents", get(handlers::ssr::my_agents_page))
        .route("/my/agents/edit", get(handlers::ssr::my_agent_edit_page))
        .route("/my/secrets", get(handlers::ssr::my_secrets_page))
        .route("/my/mcp-servers", get(handlers::ssr::my_mcp_servers_page))
        .route("/my/hooks", get(handlers::ssr::my_hooks_page))
        .route(
            "/my/versions",
            get(handlers::ssr::marketplace_versions_page),
        )
        .route("/my/activity", get(handlers::ssr::my_activity_page))
}

fn entity_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/skills", get(handlers::ssr::skills_page))
        .route("/skills/edit", get(handlers::ssr::skill_edit_page))
        .route("/skills/content", get(handlers::ssr::skills_content_page))
        .route("/skills/files", get(handlers::ssr::skills_files_page))
        .route("/skills/plugins", get(handlers::ssr::skills_plugins_page))
        .route("/skills/contexts", get(handlers::ssr::skills_contexts_page))
        .route("/agents", get(handlers::ssr::agents_page))
        .route("/agents/edit", get(handlers::ssr::agent_edit_page))
        .route("/agents/config", get(handlers::ssr::agent_config_page))
        .route("/agents/messages", get(handlers::ssr::agent_messages_page))
        .route("/agents/traces", get(handlers::ssr::agent_traces_page))
        .route("/hooks", get(handlers::ssr::hooks_page))
        .route("/hooks/edit", get(handlers::ssr::hook_edit_page))
        .route("/mcp-servers", get(handlers::ssr::mcp_servers_page))
        .route("/mcp-servers/edit", get(handlers::ssr::mcp_edit_page))
        .route("/plugins", get(handlers::ssr::plugins_page))
        .route(
            "/marketplace",
            get(handlers::ssr::marketplace_versions_page),
        )
}

fn org_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/org/plugins", get(handlers::ssr::plugins_page))
        .route("/org/skills", get(handlers::ssr::skills_page))
        .route("/org/skills/edit", get(handlers::ssr::skill_edit_page))
        .route("/org/agents", get(handlers::ssr::agents_page))
        .route("/org/agents/edit", get(handlers::ssr::agent_edit_page))
        .route("/org/mcp-servers", get(handlers::ssr::mcp_servers_page))
        .route("/org/mcp-servers/edit", get(handlers::ssr::mcp_edit_page))
        .route("/org/hooks", get(handlers::ssr::hooks_page))
        .route("/org/hooks/edit", get(handlers::ssr::hook_edit_page))
        .route("/org-marketplace", get(handlers::ssr::org_marketplace_page))
}
