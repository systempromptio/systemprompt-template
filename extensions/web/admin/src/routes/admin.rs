use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};
use sqlx::PgPool;

use super::super::gamification;
use super::super::handlers;
use super::super::middleware;

pub fn build_admin_only_routes(read_pool: &Arc<PgPool>, write_pool: &Arc<PgPool>) -> Router {
    let reads = build_admin_read_routes_inner(read_pool);
    let writes = build_admin_write_routes(write_pool);

    reads.merge(writes).layer(axum_middleware::from_fn(
        middleware::require_admin_middleware,
    ))
}

fn build_admin_read_routes_inner(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/users", get(handlers::list_users_handler))
        .route(
            "/users/{user_id}/detail",
            get(handlers::user_detail_handler),
        )
        .route("/users/{user_id}/usage", get(handlers::user_usage_handler))
        .route("/events", get(handlers::list_events_handler))
        .route("/jobs", get(handlers::list_jobs_handler))
        .route(
            "/access-control",
            get(handlers::access_control::list_access_rules_handler),
        )
        .route(
            "/access-control/departments",
            get(handlers::access_control::access_control_departments_handler),
        )
        .route(
            "/org/marketplaces",
            get(handlers::org_marketplaces::list_org_marketplaces_handler),
        )
        .with_state(Arc::clone(read_pool))
}

fn build_admin_write_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/users", post(handlers::create_user_handler))
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
        .route(
            "/marketplace-plugins/{plugin_id}/visibility",
            put(handlers::update_visibility_handler),
        )
        .route(
            "/gamification/recalculate",
            post(gamification::recalculate_handler),
        )
        .route(
            "/demo-register",
            post(handlers::demo_register::create_demo_user_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}",
            put(handlers::access_control::update_entity_rules_handler),
        )
        .route(
            "/access-control/bulk",
            put(handlers::access_control::bulk_assign_handler),
        )
        .route(
            "/org/marketplaces",
            post(handlers::org_marketplaces::create_org_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{id}",
            put(handlers::org_marketplaces::update_org_marketplace_handler)
                .delete(handlers::org_marketplaces::delete_org_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{id}/plugins",
            put(handlers::org_marketplaces::set_marketplace_plugins_handler),
        )
        .route(
            "/org/marketplaces/{id}/sync",
            post(handlers::org_marketplaces::sync_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{id}/publish",
            post(handlers::org_marketplaces::publish_marketplace_handler),
        )
        .with_state(Arc::clone(write_pool))
}

pub fn build_auth_read_routes(read_pool: &Arc<PgPool>) -> Router {
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
            "/gamification/user/{user_id}",
            get(gamification::user_gamification_handler),
        )
        .route(
            "/gamification/achievements",
            get(gamification::achievements_handler),
        )
        .route(
            "/gamification/leaderboard",
            get(gamification::leaderboard_handler),
        )
        .with_state(Arc::clone(read_pool))
}
