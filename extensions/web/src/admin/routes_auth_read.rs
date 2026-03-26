use std::sync::Arc;

use axum::routing::get;
use axum::Router;
use sqlx::PgPool;

use super::gamification;
use super::handlers;

pub(crate) fn build_auth_read_routes(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/dashboard", get(handlers::dashboard_handler))
        .route("/plugins", get(handlers::list_plugins_handler))
        .route(
            "/plugins/all-skills",
            get(handlers::list_all_skills_handler),
        )
        .route(
            "/plugins/{plugin_id}",
            get(handlers::get_plugin_detail_handler),
        )
        .route(
            "/plugins/{plugin_id}/skills",
            get(handlers::get_plugin_skills_handler),
        )
        .route(
            "/plugins/{plugin_id}/env",
            get(handlers::list_plugin_env_handler),
        )
        .route("/skills", get(handlers::list_skills_handler))
        .route("/skills/{skill_id}", get(handlers::get_skill_handler))
        .route(
            "/skills/{skill_id}/files",
            get(handlers::list_skill_files_handler),
        )
        .route(
            "/skills/{skill_id}/files/{*file_path}",
            get(handlers::get_skill_file_handler),
        )
        .route("/agents", get(handlers::list_agents_handler))
        .route("/agents/{agent_id}", get(handlers::get_agent_handler))
        .route("/mcp-servers", get(handlers::list_mcp_servers_handler))
        .route(
            "/mcp-servers/{server_id}",
            get(handlers::get_mcp_server_handler),
        )
        .route("/hooks", get(handlers::list_hooks_handler))
        .route("/hooks/{hook_id}", get(handlers::get_hook_handler))
        .route(
            "/marketplace-plugins",
            get(handlers::list_marketplace_handler),
        )
        .route(
            "/marketplace-plugins/{plugin_id}/users",
            get(handlers::marketplace_plugin_users_handler),
        )
        .route(
            "/skills/{skill_id}/base-content",
            get(handlers::marketplace_upload::get_base_skill_content_handler),
        )
        .route(
            "/marketplace-versions-summary",
            get(handlers::marketplace_upload::marketplace_all_versions_handler),
        )
        .route(
            "/gamification/leaderboard",
            get(gamification::leaderboard_handler),
        )
        .route(
            "/gamification/leaderboard/department",
            get(gamification::department_leaderboard_handler),
        )
        .route(
            "/gamification/user/{user_id}",
            get(gamification::user_gamification_handler),
        )
        .route(
            "/gamification/achievements",
            get(gamification::achievements_handler),
        )
        .route(
            "/gamification/departments",
            get(gamification::departments_handler),
        )
        .with_state(read_pool.clone())
}
