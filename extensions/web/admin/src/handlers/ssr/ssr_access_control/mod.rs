//! SSR for `/admin/access-control` — unified Access Control page.
//!
//! Three-pane layout:
//!   - Left: Departments tree (DB) with member chips.
//!   - Center: department editor or user permission matrix (matrix loads via JS).
//!   - Toolbar: source-of-truth status + "Show as YAML" + filters.

mod builders;

use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

#[derive(Debug, sqlx::FromRow)]
struct UserListRow {
    id: String,
    email: String,
    display_name: Option<String>,
    roles: Vec<String>,
    department: String,
    is_active: bool,
}

async fn fetch_users_for_tree(pool: &PgPool) -> Vec<UserListRow> {
    sqlx::query_as::<_, UserListRow>(
        r"SELECT
              u.id,
              u.email,
              COALESCE(u.display_name, u.full_name, u.name) AS display_name,
              u.roles,
              COALESCE(upe.department, '') AS department,
              (u.status = 'active') AS is_active
           FROM users u
           LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
           WHERE NOT ('anonymous' = ANY(u.roles))
             AND u.email NOT LIKE '%@anonymous.local'
           ORDER BY COALESCE(upe.department, ''), COALESCE(u.display_name, u.email)",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch users for access-control tree");
        Vec::new()
    })
}

pub async fn access_control_page(
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
        Err(r) => return *r,
    };

    let dept_stats = repositories::fetch_department_stats(&pool)
        .await
        .unwrap_or_default();
    let users = fetch_users_for_tree(&pool).await;
    let known_roles = vec!["admin", "developer", "analyst", "viewer"];

    // Lightweight catalogue of every entity admins might want to assign
    // department/role rules to. The user-permission matrix loads its own rich
    // data via the `/api/admin/access-control/users/{id}/matrix` endpoint.
    let entity_catalogue = builders::build_entity_catalogue(&services_path);

    // Department buckets: key empty/null department to "Unassigned" so it merges
    // with the synthetic group fetch_department_stats produces.
    let mut buckets: std::collections::BTreeMap<String, Vec<&UserListRow>> =
        std::collections::BTreeMap::new();
    for u in &users {
        let key = if u.department.is_empty() {
            "Unassigned".to_string()
        } else {
            u.department.clone()
        };
        buckets.entry(key).or_default().push(u);
    }
    let dept_groups: Vec<serde_json::Value> = dept_stats
        .iter()
        .map(|d| {
            let users_in: &[&UserListRow] =
                buckets.get(&d.department).map_or(&[][..], Vec::as_slice);
            json!({
                "name": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
                "users": users_in.iter().map(serialize_user).collect::<Vec<_>>(),
            })
        })
        .collect();

    let department_names: Vec<&str> = dept_stats
        .iter()
        .map(|d| d.department.as_str())
        .filter(|n| *n != "Unassigned")
        .collect();

    let data = json!({
        "page": "access-control",
        "title": "Access Control",
        "known_roles": known_roles,
        "departments": dept_groups,
        "department_names": department_names,
        "entity_catalogue": entity_catalogue,
        "stats": {
            "department_count": dept_stats.len(),
            "user_count": users.len(),
        },
    });

    super::render_page(&engine, "access-control", &data, &user_ctx, &mkt_ctx)
}

fn serialize_user(u: &&UserListRow) -> serde_json::Value {
    json!({
        "id": u.id,
        "email": u.email,
        "display_name": u.display_name.clone().unwrap_or_else(|| u.email.clone()),
        "roles": u.roles,
        "is_active": u.is_active,
    })
}
