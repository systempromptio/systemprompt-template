use std::sync::Arc;

use axum::{
    middleware as axum_middleware,
    routing::{get, patch, post, put},
    Router,
};
use sqlx::PgPool;

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
        .route("/gateway", get(handlers::get_gateway_handler))
        .route(
            "/gateway/catalog/for-user/{user_id}",
            get(handlers::gateway_catalog::for_user_handler),
        )
        .route(
            "/gateway/acl/detect",
            get(handlers::gateway_catalog::detect_handler),
        )
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
            "/access-control/users/{user_id}/matrix",
            get(handlers::access_control::user_matrix_handler),
        )
        .route(
            "/access-control/yaml-snapshot",
            get(handlers::access_control::yaml_snapshot_handler),
        )
        .route(
            "/users/roles",
            get(handlers::gateway_access::list_distinct_roles_handler),
        )
        .route(
            "/users/search",
            get(handlers::gateway_access::search_users_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/access",
            get(handlers::entity_access::list_entity_access_handler),
        )
        .route(
            "/access-control/entity-access/all",
            get(handlers::entity_access::list_all_entity_access_handler),
        )
        .route(
            "/management/departments",
            get(handlers::departments::list_departments_handler),
        )
        .with_state(Arc::clone(read_pool))
}

fn build_admin_write_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/gateway", patch(handlers::update_gateway_settings_handler))
        .route(
            "/gateway/routes",
            post(handlers::create_gateway_route_handler),
        )
        .route(
            "/gateway/routes/{idx}",
            patch(handlers::update_gateway_route_handler)
                .delete(handlers::delete_gateway_route_handler),
        )
        .route(
            "/gateway/routes/reorder",
            post(handlers::reorder_gateway_routes_handler),
        )
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
            "/access-control/entity/{entity_type}/{entity_id}/rules",
            post(handlers::entity_access::upsert_entity_rule_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/rules/{rule_id}",
            axum::routing::delete(handlers::entity_access::delete_entity_rule_handler),
        )
        .route(
            "/access-control/entity/{entity_type}/{entity_id}/default",
            patch(handlers::entity_access::set_entity_default_handler),
        )
        .route(
            "/access-control/bulk-template",
            post(handlers::entity_access::apply_template_handler),
        )
        .route(
            "/management/departments",
            post(handlers::departments::create_department_handler),
        )
        .route(
            "/management/departments/{id}",
            put(handlers::departments::update_department_handler)
                .delete(handlers::departments::delete_department_handler),
        )
        .route(
            "/management/users/{user_id}/department",
            put(handlers::departments::assign_user_to_department_handler),
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
        .route("/agents", get(handlers::list_agents_handler))
        .route("/agents/{agent_id}", get(handlers::get_agent_handler))
        .route(
            "/marketplace-plugins",
            get(handlers::list_marketplace_handler),
        )
        .with_state(Arc::clone(read_pool))
}
