//! SSR for `/admin/access-control` — unified Access Control page.
//!
//! Three-pane layout:
//!   - Left: Departments tree (DB) with member chips.
//!   - Center: department editor or user permission matrix (matrix loads via
//!     JS).
//!   - Toolbar: source-of-truth status + "Show as YAML" + filters.

mod builders;

use std::sync::Arc;

use crate::repositories;
use crate::repositories::users::access_tree::{AccessTreeUserRow, list_users_for_access_tree};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use builders::EntityCatalogue;
use serde::Serialize;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

#[derive(Debug, Serialize)]
struct SerializedUser {
    id: String,
    email: String,
    display_name: String,
    roles: Vec<String>,
    is_active: bool,
}

#[derive(Debug, Serialize)]
struct DeptGroup {
    name: String,
    user_count: i64,
    active_count: i64,
    users: Vec<SerializedUser>,
}

#[derive(Debug, Serialize)]
struct Stats {
    department_count: usize,
    user_count: usize,
}

#[derive(Debug, Serialize)]
struct AccessControlPageContext {
    page: &'static str,
    title: &'static str,
    known_roles: Vec<&'static str>,
    departments: Vec<DeptGroup>,
    department_names: Vec<String>,
    entity_catalogue: EntityCatalogue,
    stats: Stats,
}

async fn fetch_users_for_tree(pool: &PgPool) -> Vec<AccessTreeUserRow> {
    list_users_for_access_tree(pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch users for access-control tree");
        Vec::new()
    })
}

pub(crate) async fn access_control_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let dept_stats = repositories::users::user_queries::list_department_stats(&pool)
        .await
        .unwrap_or_default();
    let users = fetch_users_for_tree(&pool).await;
    let known_roles = vec!["admin", "developer", "analyst", "viewer"];

    let entity_catalogue = builders::build_entity_catalogue(&services_path);

    let mut buckets: std::collections::BTreeMap<String, Vec<&AccessTreeUserRow>> =
        std::collections::BTreeMap::new();
    for u in &users {
        let key = if u.department.is_empty() {
            "Unassigned".to_owned()
        } else {
            u.department.clone()
        };
        buckets.entry(key).or_default().push(u);
    }
    let dept_groups: Vec<DeptGroup> = dept_stats
        .iter()
        .map(|d| {
            let users_in: &[&AccessTreeUserRow] =
                buckets.get(&d.department).map_or(&[][..], Vec::as_slice);
            DeptGroup {
                name: d.department.clone(),
                user_count: d.user_count,
                active_count: d.active_count,
                users: users_in.iter().map(serialize_user).collect(),
            }
        })
        .collect();

    let department_names: Vec<String> = dept_stats
        .iter()
        .map(|d| d.department.as_str())
        .filter(|n| *n != "Unassigned")
        .map(str::to_owned)
        .collect();

    let stats = Stats {
        department_count: dept_stats.len(),
        user_count: users.len(),
    };

    let ctx = AccessControlPageContext {
        page: "access-control",
        title: "Access Control",
        known_roles,
        departments: dept_groups,
        department_names,
        entity_catalogue,
        stats,
    };

    super::render_typed_page(&engine, "access-control", &ctx, &user_ctx, &mkt_ctx)
}

fn serialize_user(u: &&AccessTreeUserRow) -> SerializedUser {
    SerializedUser {
        id: u.id.clone(),
        email: u.email.clone(),
        display_name: u.display_name.clone().unwrap_or_else(|| u.email.clone()),
        roles: u.roles.clone(),
        is_active: u.is_active,
    }
}
