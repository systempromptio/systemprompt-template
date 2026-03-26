use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, post, put},
    Router,
};
use sqlx::PgPool;

use super::gamification;
use super::handlers;
use super::middleware;

pub(crate) fn build_admin_only_routes(read_pool: &Arc<PgPool>, write_pool: &Arc<PgPool>) -> Router {
    let reads = Router::new()
        .route("/users", get(handlers::list_users_handler))
        .route(
            "/users/{user_id}/detail",
            get(handlers::user_detail_handler),
        )
        .route("/users/{user_id}/usage", get(handlers::user_usage_handler))
        .route("/events", get(handlers::list_events_handler))
        .route("/jobs", get(handlers::list_jobs_handler))
        .route(
            "/access-control/rules",
            get(handlers::access_control::list_access_rules_handler),
        )
        .route(
            "/access-control/departments",
            get(handlers::access_control::access_control_departments_handler),
        )
        .with_state(read_pool.clone());

    let writes = Router::new()
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
            "/access-control/entity/{entity_type}/{entity_id}",
            put(handlers::access_control::update_entity_rules_handler),
        )
        .route(
            "/access-control/bulk",
            put(handlers::access_control::bulk_assign_handler),
        )
        .route(
            "/org/marketplaces",
            get(handlers::org_marketplaces::list_org_marketplaces_handler)
                .post(handlers::org_marketplaces::create_org_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{marketplace_id}",
            put(handlers::org_marketplaces::update_org_marketplace_handler)
                .delete(handlers::org_marketplaces::delete_org_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{marketplace_id}/plugins",
            put(handlers::org_marketplaces::set_marketplace_plugins_handler),
        )
        .route(
            "/org/marketplaces/{marketplace_id}/sync",
            post(handlers::org_marketplaces::sync_marketplace_handler),
        )
        .route(
            "/org/marketplaces/{marketplace_id}/publish",
            post(handlers::org_marketplaces::publish_marketplace_handler),
        )
        .route(
            "/gamification/recalculate",
            post(gamification::recalculate_handler),
        )
        .with_state(write_pool.clone());

    reads.merge(writes).layer(axum_middleware::from_fn(
        middleware::require_admin_middleware,
    ))
}
