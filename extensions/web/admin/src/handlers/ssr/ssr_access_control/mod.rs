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
use crate::repositories::users_grp::access_tree::{AccessTreeUserRow, list_users_for_access_tree};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

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
        Err(r) => return *r,
    };

    let dept_stats = repositories::fetch_department_stats(&pool)
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
    let dept_groups: Vec<serde_json::Value> = dept_stats
        .iter()
        .map(|d| {
            let users_in: &[&AccessTreeUserRow] =
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

fn serialize_user(u: &&AccessTreeUserRow) -> serde_json::Value {
    json!({
        "id": u.id,
        "email": u.email,
        "display_name": u.display_name.clone().unwrap_or_else(|| u.email.clone()),
        "roles": u.roles,
        "is_active": u.is_active,
    })
}
