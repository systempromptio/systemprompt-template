//! Group and project routes, split by the role tier that may reach them.
//!
//! Six builders rather than two because the tiers differ per verb: reads are
//! open to the console roles, ordinary writes to the two admin roles, and the
//! AD-mapping routes to `platform_admin` alone — a mapping decides what the
//! directory grants, so it sits at the same tier as granting `platform_admin`
//! itself.

use std::sync::Arc;

use axum::Router;
use axum::routing::{delete, get, post, put};
use sqlx::PgPool;

use super::super::handlers::{groups, projects};

pub(crate) fn group_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/groups", get(groups::list_groups_handler))
        .route("/groups/{group_id}", get(groups::get_group_handler))
        .route(
            "/groups/{group_id}/members",
            get(groups::members::list_group_members_handler),
        )
        .route(
            "/groups/{group_id}/ad-mappings",
            get(groups::mappings::list_group_ad_mappings_handler),
        )
        .route(
            "/groups/{group_id}/marketplaces",
            get(groups::marketplaces::list_group_marketplaces_handler),
        )
        .route(
            "/groups/{group_id}/usage",
            get(groups::usage::get_group_usage_handler),
        )
}

pub(crate) fn group_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/groups", post(groups::create_group_handler))
        .route(
            "/groups/{group_id}",
            put(groups::update_group_handler).delete(groups::delete_group_handler),
        )
        .route(
            "/groups/{group_id}/members",
            post(groups::members::add_group_member_handler),
        )
        .route(
            "/groups/{group_id}/members/{user_id}",
            delete(groups::members::remove_group_member_handler),
        )
        .route(
            "/groups/{group_id}/marketplaces",
            put(groups::marketplaces::set_group_marketplaces_handler),
        )
}

pub(crate) fn group_platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/groups/{group_id}/ad-mappings",
            post(groups::mappings::add_group_ad_mapping_handler),
        )
        .route(
            "/groups/{group_id}/ad-mappings/{ad_group}",
            delete(groups::mappings::delete_group_ad_mapping_handler),
        )
}

pub(crate) fn project_read_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/projects", get(projects::list_projects_handler))
        .route("/projects/{project_id}", get(projects::get_project_handler))
        .route(
            "/projects/{project_id}/members",
            get(projects::members::list_project_members_handler),
        )
        .route(
            "/projects/{project_id}/ad-mappings",
            get(projects::mappings::list_project_ad_mappings_handler),
        )
        .route(
            "/projects/{project_id}/usage",
            get(projects::usage::get_project_usage_handler),
        )
}

pub(crate) fn project_write_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route("/projects", post(projects::create_project_handler))
        .route(
            "/projects/{project_id}",
            put(projects::update_project_handler).delete(projects::delete_project_handler),
        )
        .route(
            "/projects/{project_id}/members",
            post(projects::members::add_project_member_handler),
        )
        .route(
            "/projects/{project_id}/members/{user_id}",
            delete(projects::members::remove_project_member_handler),
        )
}

pub(crate) fn project_platform_routes() -> Router<Arc<PgPool>> {
    Router::new()
        .route(
            "/projects/{project_id}/ad-mappings",
            post(projects::mappings::add_project_ad_mapping_handler),
        )
        .route(
            "/projects/{project_id}/ad-mappings/{ad_group}",
            delete(projects::mappings::delete_project_ad_mapping_handler),
        )
}
