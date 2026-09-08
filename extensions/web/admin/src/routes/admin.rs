//! Admin JSON API routes.
//!
//! Read and write routes are built against separate pools so read traffic can
//! be directed at a replica.

use std::sync::Arc;

use axum::routing::{delete, get, patch, post, put};
use axum::{Router, middleware as axum_middleware};
use sqlx::PgPool;

use super::super::{handlers, middleware};

pub(crate) fn build_admin_only_routes(read_pool: &Arc<PgPool>, write_pool: &Arc<PgPool>) -> Router {
    let reads = build_admin_read_routes_inner(read_pool)
        .merge(dashboard_reads(read_pool))
        .layer(axum_middleware::from_fn_with_state(
            crate::types::ROLES_CONSOLE,
            middleware::require_roles_middleware,
        ));
    let writes = build_admin_write_routes(write_pool)
        .merge(dashboard_writes(write_pool))
        .layer(axum_middleware::from_fn_with_state(
            crate::types::ROLES_MANAGE,
            middleware::require_roles_middleware,
        ));
    let platform = admin_groups::group_platform_routes()
        .merge(admin_groups::project_platform_routes())
        .with_state(Arc::clone(write_pool))
        .layer(axum_middleware::from_fn_with_state(
            crate::types::ROLES_PLATFORM,
            middleware::require_roles_middleware,
        ));
    reads.merge(writes).merge(platform)
}

fn build_admin_read_routes_inner(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/gateway", get(handlers::get_gateway_handler))
        .route(
            "/gateway/catalog/for-user/{user_id}",
            get(handlers::gateway_catalog::for_user_handler),
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

fn build_gateway_write_routes() -> Router<Arc<PgPool>> {
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
}

fn build_admin_write_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        // Why: the legacy GET endpoint emits audit events, so it requires the primary pool and
        // admin tier.
        .route(
            "/gateway/acl/detect",
            get(handlers::gateway_catalog::detect_handler),
        )
        .merge(build_gateway_write_routes())
        .route("/users", post(handlers::create_user_handler))
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
        .route(
            "/users/{user_id}/share-token",
            post(handlers::share::issue_share_token_handler),
        )
        .route(
            "/users/{user_id}/slack-identity",
            post(handlers::slack_identity::link_slack_identity_handler)
                .delete(handlers::slack_identity::unlink_slack_identity_handler),
        )
        .route(
            "/users/{user_id}/pats",
            post(handlers::access_tokens::issue_user_pat),
        )
        .route(
            "/users/{user_id}/pats/{id}",
            delete(handlers::access_tokens::revoke_user_pat),
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
            delete(handlers::entity_access::delete_entity_rule_handler),
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

pub(crate) fn build_auth_read_routes(read_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route("/dashboard", get(handlers::dashboard_handler))
        .route("/plugins", get(handlers::list_plugins_handler))
        .route(
            "/plugins/{plugin_id}/env",
            get(handlers::list_plugin_env_handler),
        )
        .route("/agents", get(handlers::list_agents_handler))
        .route("/agents/{agent_id}", get(handlers::get_agent_handler))
        .with_state(Arc::clone(read_pool))
}

use super::admin_groups;

fn dashboard_reads(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/users/{user_id}/roles",
            get(handlers::roles::get_user_roles_handler),
        )
        .route(
            "/users/{user_id}/scope-defaults",
            get(handlers::scope_defaults::get_user_scope_defaults_handler),
        )
        .route(
            "/users/{user_id}/sessions",
            get(handlers::list_user_sessions_handler),
        )
        .merge(admin_groups::group_read_routes())
        .merge(admin_groups::project_read_routes())
        .with_state(Arc::clone(pool))
}

fn dashboard_writes(pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/users/{user_id}/sessions",
            delete(handlers::revoke_all_user_sessions_handler),
        )
        .route(
            "/users/{user_id}/salesforce-identity",
            post(handlers::salesforce_identity::link_salesforce_identity_handler)
                .delete(handlers::salesforce_identity::unlink_salesforce_identity_handler),
        )
        .route(
            "/users/{user_id}/sessions/{session_id}",
            delete(handlers::revoke_user_session_handler),
        )
        .route(
            "/users/{user_id}/roles",
            put(handlers::roles::set_user_roles_handler),
        )
        .route(
            "/users/{user_id}/scope-defaults",
            put(handlers::scope_defaults::set_user_scope_defaults_handler),
        )
        .route(
            "/scope-defaults/recompute",
            post(handlers::scope_defaults::recompute_scope_defaults_handler),
        )
        .route(
            "/management/devices",
            post(handlers::devices::enroll_device),
        )
        .route(
            "/devices/{kind}/{id}",
            delete(handlers::devices::admin_revoke_credential),
        )
        .route(
            "/approvals/{call_id}/approve",
            post(handlers::approvals::approve_handler),
        )
        .route(
            "/approvals/{call_id}/deny",
            post(handlers::approvals::deny_handler),
        )
        .merge(admin_groups::group_write_routes())
        .merge(admin_groups::project_write_routes())
        .with_state(Arc::clone(pool))
}

pub(crate) fn build_self_service_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .route(
            "/user/settings",
            put(handlers::self_service::update_own_settings_handler),
        )
        .route(
            "/user/account",
            delete(handlers::self_service::delete_own_account_handler),
        )
        .with_state(Arc::clone(write_pool))
}
