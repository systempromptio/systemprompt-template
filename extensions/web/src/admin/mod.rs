pub mod activity;
pub mod event_hub;
pub mod gamification;
pub(crate) mod handlers;
mod middleware;
pub mod numeric;
pub mod repositories;
mod routes;
pub mod slack_alerts;
pub mod templates;
pub mod tier_enforcement;
pub mod tier_limits;
pub mod types;

use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use templates::AdminTemplateEngine;

pub use types::{CreateUserRequest, MarketplaceContext, UsageEvent, UserContext, UserSummary};

pub fn hooks_webhook_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/hooks/track",
            post(handlers::hooks_track::handle_hook_track),
        )
        .route(
            "/hooks/govern",
            post(handlers::govern_tool_use),
        )
        .route(
            "/hooks/statusline",
            post(handlers::track_statusline_event),
        )
        .route(
            "/hooks/transcript",
            post(handlers::track_transcript_event),
        )
        .layer(Extension(event_hub::EventHub::default()))
        .layer(Extension(None::<Arc<systemprompt::ai::AiService>>))
        .layer(Extension(tier_enforcement::TierEnforcementCache::default()))
        .with_state(pool)
}

pub fn secrets_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/api/v1/secrets/{plugin_id}/token",
            post(handlers::secrets::create_resolution_token_handler),
        )
        .route(
            "/api/v1/secrets/{plugin_id}/resolve",
            get(handlers::secrets::resolve_secrets_handler),
        )
        .route(
            "/admin/api/secrets/{plugin_id}/audit",
            get(handlers::secrets::audit_log_handler),
        )
        .route(
            "/admin/api/secrets/{plugin_id}/rotate",
            post(handlers::secrets::rotate_handler),
        )
        .with_state(pool)
}

pub fn marketplace_git_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/marketplace/{user_id}",
            get(handlers::marketplace_json::marketplace_json_handler)
                .post(handlers::marketplace_upload::marketplace_upload_handler),
        )
        .route(
            "/marketplace/{user_id}/versions",
            get(handlers::marketplace_upload::marketplace_versions_handler),
        )
        .route(
            "/marketplace/{user_id}/versions/{version_id}",
            get(handlers::marketplace_upload::marketplace_version_detail_handler),
        )
        .route(
            "/marketplace/{user_id}/changelog",
            get(handlers::marketplace_upload::marketplace_changelog_handler),
        )
        .route(
            "/marketplace/{user_id}/restore/{version_id}",
            post(handlers::marketplace_upload::marketplace_restore_handler),
        )
        .route(
            "/marketplace/{user_id}/export/marketplace.zip",
            get(handlers::export_zip::export_marketplace_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/export/cowork.zip",
            get(handlers::export_zip::export_cowork_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/export/plugin/{plugin_id}",
            get(handlers::export_zip::export_plugin_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/cowork/{*path}",
            get(handlers::marketplace_git::cowork_git_handler)
                .post(handlers::marketplace_git::cowork_upload_pack_handler),
        )
        .route(
            "/marketplace/{user_id}/cowork.git/{*path}",
            get(handlers::marketplace_git::cowork_git_handler)
                .post(handlers::marketplace_git::cowork_upload_pack_handler),
        )
        .route(
            "/marketplace/{user_id}/{*path}",
            get(handlers::marketplace_git::marketplace_git_handler)
                .post(handlers::marketplace_git::git_upload_pack_handler),
        )
        .with_state(pool)
}

pub fn workspace_ssr_router(
    pool: Arc<PgPool>,
    engine: AdminTemplateEngine,
    hub: event_hub::EventHub,
    ai_service: Option<Arc<systemprompt::ai::AiService>>,
    tier_cache: tier_enforcement::TierEnforcementCache,
) -> Router {
    let inner = Router::new()
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
            pool.clone(),
            middleware::user_context_middleware,
        ))
        .with_state(pool);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(inner),
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
        .route(
            "/org-marketplace",
            get(handlers::ssr::org_marketplace_page),
        )
}

pub fn admin_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = Router::new()
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
        .route("/users", get(handlers::ssr::users_page))
        .route("/jobs", get(handlers::ssr::jobs_page))
        .route("/events", get(handlers::ssr::events_page))
        .route("/profile", get(handlers::ssr::profile_page))
        .route("/settings", get(handlers::ssr::settings_page))
        .route("/achievements", get(handlers::ssr::achievements_page))
        .route("/leaderboard", get(handlers::ssr::leaderboard_page))
        .route("/user", get(handlers::ssr::user_detail_page))
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
        .route("/governance", get(handlers::ssr::governance_page))
        .route("/traces", get(handlers::ssr::traces_page))
        .route("/access-control", get(handlers::ssr::access_control_page))
        .route("/skills", get(handlers::ssr::skills_page))
        .route("/skills/edit", get(handlers::ssr::skill_edit_page))
        .route("/agents", get(handlers::ssr::agents_page))
        .route("/agents/edit", get(handlers::ssr::agent_edit_page))
        .route("/hooks", get(handlers::ssr::hooks_page))
        .route("/hooks/edit", get(handlers::ssr::hook_edit_page))
        .route("/mcp-servers", get(handlers::ssr::mcp_servers_page))
        .route("/mcp-servers/edit", get(handlers::ssr::mcp_edit_page))
        .route("/plugins", get(handlers::ssr::plugins_page))
        .route("/marketplace", get(handlers::ssr::marketplace_versions_page))
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
            pool.clone(),
            middleware::user_context_middleware,
        ))
        .with_state(pool.clone());

    let combined = Router::new()
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
        .layer(Extension(engine))
        .with_state(pool)
        .fallback_service(inner);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(combined),
    )
}

pub fn admin_router(
    read_pool: Arc<PgPool>,
    write_pool: Arc<PgPool>,
    tier_cache: tier_enforcement::TierEnforcementCache,
) -> Router {
    let admin_only = routes::build_admin_only_routes(&read_pool, &write_pool);
    let auth_reads = routes::build_auth_read_routes(&read_pool);
    let auth_writes = routes::build_auth_write_routes(write_pool);

    admin_only
        .merge(auth_reads)
        .merge(auth_writes)
        .layer(Extension(tier_cache))
        .layer(axum_middleware::from_fn(
            middleware::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            read_pool,
            middleware::user_context_middleware,
        ))
}
