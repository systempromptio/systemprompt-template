//! Admin JSON API routes.
//!
//! Read and write routes are built against separate pools so read traffic can
//! be directed at a replica. That split is also the authorisation boundary,
//! and it has three tiers: reads open to any console role, writes to the two
//! admin roles, and a platform tier holding the controls that decide what the
//! directory itself grants. One shared admin layer would have been simpler,
//! but every write here — user create/update/delete, session and identity
//! mutation, device enrolment, access-control rules, gateway configuration —
//! relies on that layer for its only authorisation check, so widening it to
//! the semi-admin role would have handed a project manager the whole control
//! plane. The boundary lives in the router where it can be read off in one
//! place rather than in thirty handler bodies.

use std::sync::Arc;

use axum::routing::{delete, get, patch, post, put};
use axum::{Router, middleware as axum_middleware};
use sqlx::PgPool;

use super::super::types::{ROLES_CONSOLE, ROLES_MANAGE, ROLES_PLATFORM};
use super::super::{handlers, middleware};
use super::admin_groups;

mod read;
use read::build_admin_read_routes_inner;

pub(crate) fn build_admin_only_routes(
    read_pool: &Arc<PgPool>,
    write_pool: &Arc<PgPool>,
    _owner: systemprompt::identifiers::UserId,
) -> Router {
    // Why: the split is the `project_manager` boundary. Reads are the admin
    // dashboard's data and open to any console role; every ordinary write
    // mutates an identity, a role, an ACL rule or the gateway config, so it
    // stays with the admin roles. The platform tier is narrower still: an AD
    // mapping decides what the directory grants everyone, so only
    // `platform_admin` may move one.
    let reads = build_admin_read_routes_inner(read_pool).layer(
        axum_middleware::from_fn_with_state(ROLES_CONSOLE, middleware::require_roles_middleware),
    );
    let writes = build_admin_write_routes(write_pool).layer(axum_middleware::from_fn_with_state(
        ROLES_MANAGE,
        middleware::require_roles_middleware,
    ));
    let platform = build_admin_platform_routes(write_pool).layer(
        axum_middleware::from_fn_with_state(ROLES_PLATFORM, middleware::require_roles_middleware),
    );

    reads.merge(writes).merge(platform)
}

// Why: the access-control writes are their own table — every route here edits
// a rule row, and none of them is reachable without the console role.
fn build_access_control_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
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
        .route(
            "/users",
            post(handlers::users_bootstrap::create_user_handler),
        )
        .route(
            "/users/{user_id}",
            put(handlers::update_user_handler).delete(handlers::delete_user_handler),
        )
        .route(
            "/users/{user_id}/share-token",
            post(handlers::share::issue_share_token_handler),
        )
        // Why: set administratively because ADFS replaced the Salesforce SSO
        // login that used to capture the Username from a `preferred_username`
        // claim. Nothing derives it now, so someone has to state it.
        .route(
            "/users/{user_id}/salesforce-identity",
            post(handlers::salesforce_identity::link_salesforce_identity_handler)
                .delete(handlers::salesforce_identity::unlink_salesforce_identity_handler),
        )
        .route(
            "/connectors/{provider}/reprovision",
            post(handlers::connector_reprovision::reprovision_connector_handler),
        )
        .route(
            "/users/{user_id}/sessions",
            get(handlers::list_user_sessions_handler)
                .delete(handlers::revoke_all_user_sessions_handler),
        )
        .route(
            "/users/{user_id}/sessions/{session_id}",
            delete(handlers::revoke_user_session_handler),
        )
        .merge(build_access_control_write_routes())
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
        .merge(admin_groups::group_write_routes())
        .merge(admin_groups::project_write_routes())
        .merge(build_management_write_routes())
        .merge(build_approval_write_routes())
        .with_state(Arc::clone(write_pool))
}

// Why: the platform tier is empty of ordinary routes on purpose. It carries
// only the directory-shaped controls, which are the AD mappings; everything
// else an admin may drive belongs one tier down.
fn build_admin_platform_routes(write_pool: &Arc<PgPool>) -> Router {
    Router::new()
        .merge(admin_groups::group_platform_routes())
        .merge(admin_groups::project_platform_routes())
        .with_state(Arc::clone(write_pool))
}

// Why: the management surface (devices) is split out so the
// gateway/user/access-control routes above stay readable as one list; both
// halves are mounted on the same write pool. A user's group and project
// membership is not settable here: the directory-sourced half is the AD
// group, re-projected at every sign-in, and the manual half has its own
// routes.
fn build_management_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/management/devices",
            post(handlers::devices::enroll_device),
        )
        // Why: `{kind}` rather than two routes because the two credentials
        // differ only in which table holds them; the fleet page revokes both
        // through one button and one confirmation.
        .route(
            "/devices/{kind}/{id}",
            delete(handlers::devices::admin_revoke_credential),
        )
}

// Why: two routes rather than one taking a verdict in the body. The verb is the
// entire payload of an approval decision, and a route per verb means a typo
// cannot reach a 200 — it 404s at the router before any row is touched.
fn build_approval_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/approvals/{call_id}/approve",
            post(handlers::approvals::approve_handler),
        )
        .route(
            "/approvals/{call_id}/deny",
            post(handlers::approvals::deny_handler),
        )
}

// Why: a tier of its own. Every other write under `/api/public/admin` is
// behind `ROLES_MANAGE` because it mutates somebody else's identity or the
// instance's configuration; these two mutate only the caller's own account and
// take their target from the validated session, so gating them on an admin
// role would mean the settings form worked for nobody but administrators.
// They need the write pool, which is why they are not in
// `build_auth_read_routes`.
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
