use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

pub(crate) async fn org_marketplace_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let marketplace_id = params.get("id");
    let is_edit = marketplace_id.is_some();

    let marketplace = if let Some(id) = marketplace_id {
        match repositories::org_marketplaces::get_org_marketplace(&pool, id).await {
            Ok(m) => m,
            Err(e) => return internal_error("Failed to fetch marketplace", e),
        }
    } else {
        None
    };

    let marketplace_plugin_ids = if let Some(id) = marketplace_id {
        match repositories::org_marketplaces::list_marketplace_plugin_ids(&pool, id).await {
            Ok(v) => v,
            Err(e) => return internal_error("Failed to list marketplace plugins", e),
        }
    } else {
        vec![]
    };

    let admin_roles = vec!["admin".to_string()];
    let all_plugins = match repositories::list_plugins_for_roles(&services_path, &admin_roles) {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to list plugins", e),
    };

    let plugins_list: Vec<serde_json::Value> = all_plugins
        .iter()
        .map(|p| {
            json!({
                "value": p.id,
                "name": p.name,
                "checked": marketplace_plugin_ids.contains(&p.id),
            })
        })
        .collect();

    let known_roles = match repositories::fetch_distinct_roles(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to fetch roles", e),
    };
    let entity_rules = if let Some(id) = marketplace_id {
        match repositories::access_control::list_rules_for_entity(&pool, "marketplace", id).await {
            Ok(v) => v,
            Err(e) => return internal_error("Failed to fetch access control rules", e),
        }
    } else {
        vec![]
    };

    let roles_list: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|role_name| {
            let checked = entity_rules.iter().any(|r| {
                r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
            });
            json!({ "value": role_name, "checked": checked })
        })
        .collect();

    let departments = match repositories::fetch_department_stats(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to fetch department stats", e),
    };

    let departments_list: Vec<serde_json::Value> = departments
        .iter()
        .filter(|dept| dept.department != "Unassigned")
        .map(|dept| {
            let rule = entity_rules
                .iter()
                .find(|r| r.rule_type == "department" && r.rule_value == dept.department);
            let checked = rule.is_some_and(|r| r.access == "allow");
            let default_included = rule.is_some_and(|r| r.default_included);
            json!({
                "value": dept.department,
                "name": dept.department,
                "checked": checked,
                "default_included": default_included,
                "user_count": dept.user_count,
            })
        })
        .collect();

    let data = json!({
        "page": "org-marketplace-edit",
        "title": if is_edit { "Edit Marketplace" } else { "Create Marketplace" },
        "is_edit": is_edit,
        "marketplace": marketplace,
        "plugins_list": plugins_list,
        "roles_list": roles_list,
        "departments_list": departments_list,
    });

    super::render_page(&engine, "org-marketplace-edit", &data, &user_ctx, &mkt_ctx)
}

fn internal_error(msg: &str, err: impl std::fmt::Display) -> Response {
    tracing::error!(error = %err, "{msg}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(format!(
            "<h1>Internal Server Error</h1><p>{msg}: {err}</p>"
        )),
    )
        .into_response()
}
