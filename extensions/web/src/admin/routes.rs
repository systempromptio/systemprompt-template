use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Extension, Router,
};
use sqlx::PgPool;
use tower_http::normalize_path::NormalizePathLayer;

use super::handlers;
use super::middleware;
use super::templates::AdminTemplateEngine;

use super::routes_admin::build_admin_only_routes;
use super::routes_auth_read::build_auth_read_routes;
use super::routes_auth_write::build_auth_write_routes;

pub fn hooks_webhook_router(pool: Arc<PgPool>) -> Router {
    Router::new()
        .route("/hooks/track", post(handlers::track_hook_event))
        .route("/hooks/govern", post(handlers::govern_tool_use))
        .route("/hooks/statusline", post(handlers::track_statusline_event))
        .route(
            "/hooks/transcript",
            post(handlers::track_transcript_event),
        )
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
            "/marketplace/org/{marketplace_id}",
            get(handlers::marketplace_git::org_marketplace_json_handler),
        )
        .route(
            "/marketplace/org/{marketplace_id}/{*path}",
            get(handlers::marketplace_git::org_marketplace_git_handler)
                .post(handlers::marketplace_git::org_git_upload_pack_handler),
        )
        .route(
            "/marketplace/{user_id}",
            get(handlers::marketplace_git::marketplace_json_handler)
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
            "/marketplace/{user_id}/export/plugin/{plugin_id}",
            get(handlers::export_zip::export_plugin_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/export/marketplace",
            get(handlers::export_zip::export_marketplace_zip_handler),
        )
        .route(
            "/marketplace/{user_id}/{*path}",
            get(handlers::marketplace_git::marketplace_git_handler)
                .post(handlers::marketplace_git::git_upload_pack_handler),
        )
        .route(
            "/auth/magic-link",
            post(handlers::magic_link::request_magic_link),
        )
        .route(
            "/auth/magic-link/validate",
            post(handlers::magic_link::validate_magic_link),
        )
        .with_state(pool)
}

pub fn admin_ssr_router(pool: Arc<PgPool>, engine: AdminTemplateEngine) -> Router {
    let inner = Router::new()
        .route("/", get(handlers::ssr::dashboard_page))
        .route("/api/sse/dashboard", get(handlers::sse::dashboard_sse))
        .route("/skills", get(handlers::ssr::skills_page))
        .route("/agents", get(handlers::ssr::agents_page))
        .route("/hooks", get(handlers::ssr::hooks_page))
        .route("/mcp-servers", get(handlers::ssr::mcp_servers_page))
        .route("/skills/edit", get(handlers::ssr::skill_edit_page))
        .route("/agents/edit", get(handlers::ssr::agent_edit_page))
        .route("/hooks/edit", get(handlers::ssr::hook_edit_page))
        .route("/mcp-servers/edit", get(handlers::ssr::mcp_edit_page))
        .route("/users", get(handlers::ssr::users_page))
        .route("/jobs", get(handlers::ssr::jobs_page))
        .route("/events", get(handlers::ssr::events_page))
        .route("/access-control", get(handlers::ssr::access_control_page))
        .route("/leaderboard", get(handlers::ssr::leaderboard_page))
        .route("/achievements", get(handlers::ssr::achievements_page))
        .route("/user", get(handlers::ssr::user_detail_page))
        .route("/plugins", get(handlers::ssr::plugins_page))
        .route("/plugins/edit", get(handlers::ssr::plugin_edit_page))
        .route("/plugins/create", get(handlers::ssr::plugin_create_page))
        .route("/marketplace", get(handlers::ssr::marketplace_page))
        .route(
            "/marketplace-versions",
            get(handlers::ssr::marketplace_versions_page),
        )
        .route(
            "/org/marketplaces",
            get(handlers::ssr::org_marketplaces_page),
        )
        .route(
            "/org/marketplaces/edit",
            get(handlers::ssr::org_marketplace_edit_page),
        )
        .route("/org/plugins", get(handlers::ssr::plugins_page))
        .route("/org/plugins/edit", get(handlers::ssr::plugin_edit_page))
        .route(
            "/org/plugins/create",
            get(handlers::ssr::plugin_create_page),
        )
        .route("/org/skills", get(handlers::ssr::skills_page))
        .route("/org/skills/edit", get(handlers::ssr::skill_edit_page))
        .route("/org/agents", get(handlers::ssr::agents_page))
        .route("/org/agents/edit", get(handlers::ssr::agent_edit_page))
        .route("/org/mcp-servers", get(handlers::ssr::mcp_servers_page))
        .route("/org/mcp-servers/edit", get(handlers::ssr::mcp_edit_page))
        .route("/org/hooks", get(handlers::ssr::hooks_page))
        .route("/org/hooks/edit", get(handlers::ssr::hook_edit_page))
        .route("/browse-plugins", get(handlers::ssr::browse_plugins_page))
        .route("/add-passkey", get(handlers::ssr::add_passkey_page))
        .route("/my/marketplace", get(handlers::ssr::my_marketplace_page))
        .route("/my/plugins", get(handlers::ssr::my_plugins_page))
        .route("/my/plugins/edit", get(handlers::ssr::my_plugin_edit_page))
        .route("/my/skills", get(handlers::ssr::my_skills_page))
        .route("/my/skills/edit", get(handlers::ssr::my_skill_edit_page))
        .route("/my/agents", get(handlers::ssr::my_agents_page))
        .route("/my/agents/edit", get(handlers::ssr::my_agent_edit_page))
        .route("/my/mcp-servers", get(handlers::ssr::my_mcp_servers_page))
        .route("/my/mcp-servers/edit", get(handlers::ssr::my_mcp_edit_page))
        .route("/my/hooks", get(handlers::ssr::my_hooks_page))
        .route("/my/hooks/edit", get(handlers::ssr::my_hook_edit_page))
        .route("/my/secrets", get(handlers::ssr::my_secrets_page))
        .route(
            "/my/plugins/view",
            get(handlers::ssr::my_plugin_view_page),
        )
        .route(
            "/my/versions",
            get(handlers::ssr::marketplace_versions_page),
        )
        .route("/auth/me", get(middleware::auth_me_handler))
        .layer(Extension(engine.clone()))
        .layer(axum_middleware::from_fn(
            middleware::marketplace_context_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            pool.clone(),
            middleware::user_context_middleware,
        ))
        .with_state(pool);

    let combined = Router::new()
        .route("/login", get(handlers::ssr::login_page))
        .layer(Extension(engine))
        .fallback_service(inner);

    Router::new().fallback_service(
        tower::ServiceBuilder::new()
            .layer(NormalizePathLayer::trim_trailing_slash())
            .service(combined),
    )
}

pub fn admin_router(read_pool: &Arc<PgPool>, write_pool: Arc<PgPool>) -> Router {
    let admin_only = build_admin_only_routes(read_pool, &write_pool);
    let auth_reads = build_auth_read_routes(read_pool);
    let auth_writes = build_auth_write_routes(write_pool.clone());

    admin_only
        .merge(auth_reads)
        .merge(auth_writes)
        .layer(axum_middleware::from_fn(
            middleware::require_auth_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            write_pool,
            middleware::user_context_middleware,
        ))
}
