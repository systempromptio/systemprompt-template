use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

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

#[allow(clippy::too_many_lines)]
pub(crate) async fn org_marketplaces_page(
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

    let marketplaces = match repositories::org_marketplaces::list_org_marketplaces(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to list org marketplaces", e),
    };

    let all_rules = match repositories::access_control::list_all_rules(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to fetch access control rules", e),
    };

    let departments = match repositories::fetch_department_stats(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to fetch department stats", e),
    };

    let known_roles = match repositories::fetch_distinct_roles(&pool).await {
        Ok(v) => v,
        Err(e) => return internal_error("Failed to fetch roles", e),
    };

    let mut rules_map: HashMap<
        (String, String),
        Vec<&crate::admin::types::access_control::AccessControlRule>,
    > = HashMap::new();
    for rule in &all_rules {
        if rule.entity_type == "marketplace" {
            rules_map
                .entry((rule.entity_type.clone(), rule.entity_id.clone()))
                .or_default()
                .push(rule);
        }
    }

    let dept_names: Vec<&str> = departments
        .iter()
        .filter(|d| d.department != "Unassigned")
        .map(|d| d.department.as_str())
        .collect();

    let all_dept_names: Vec<serde_json::Value> = dept_names
        .iter()
        .map(|name| json!({ "name": name }))
        .collect();

    let all_departments_detail: Vec<serde_json::Value> = departments
        .iter()
        .filter(|d| d.department != "Unassigned")
        .map(|d| {
            json!({
                "value": d.department,
                "user_count": d.user_count,
            })
        })
        .collect();

    let all_roles_json: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|r| json!({ "value": r }))
        .collect();

    let mut marketplaces_json: Vec<serde_json::Value> = Vec::new();
    let mut total_plugins = 0usize;
    let mut total_skills = 0usize;

    for mkt in &marketplaces {
        let plugin_ids = match repositories::org_marketplaces::list_marketplace_plugin_ids(
            &pool, &mkt.id,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, marketplace_id = %mkt.id, "Failed to list plugin IDs");
                vec![]
            }
        };

        let mut skill_count = 0usize;
        let mut agent_count = 0usize;
        let mut mcp_count = 0usize;
        let mut plugins_detail: Vec<serde_json::Value> = Vec::new();

        for pid in &plugin_ids {
            if let Ok(Some(detail)) = repositories::get_plugin_detail(&services_path, pid) {
                let sc = detail.skills.len();
                let ac = detail.agents.len();
                let mc = detail.mcp_servers.len();
                skill_count += sc;
                agent_count += ac;
                mcp_count += mc;
                plugins_detail.push(json!({
                    "id": detail.id,
                    "name": detail.name,
                    "description": detail.description,
                    "category": detail.category,
                    "skill_count": sc,
                    "agent_count": ac,
                    "mcp_count": mc,
                    "hook_count": 0,
                }));
            } else {
                plugins_detail.push(json!({
                    "id": pid,
                    "name": pid,
                    "description": "",
                    "category": "",
                    "skill_count": 0,
                    "agent_count": 0,
                    "mcp_count": 0,
                    "hook_count": 0,
                }));
            }
        }

        total_plugins += plugin_ids.len();
        total_skills += skill_count;

        let key = ("marketplace".to_string(), mkt.id.clone());
        let entity_rules = rules_map.get(&key);

        let mut role_count = 0;
        let roles: Vec<serde_json::Value> = known_roles
            .iter()
            .map(|role_name| {
                let assigned = entity_rules.is_some_and(|rules| {
                    rules.iter().any(|r| {
                        r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
                    })
                });
                if assigned {
                    role_count += 1;
                }
                json!({ "name": role_name, "assigned": assigned })
            })
            .collect();

        let mut dept_count = 0;
        let dept_assignments: Vec<serde_json::Value> = dept_names
            .iter()
            .map(|dept_name| {
                let rule = entity_rules.and_then(|rules| {
                    rules
                        .iter()
                        .find(|r| r.rule_type == "department" && r.rule_value == *dept_name)
                });
                let assigned = rule.is_some_and(|r| r.access == "allow");
                let default_included = rule.is_some_and(|r| r.default_included);
                if assigned {
                    dept_count += 1;
                }
                json!({
                    "name": dept_name,
                    "assigned": assigned,
                    "default_included": default_included,
                })
            })
            .collect();

        let latest_sync = repositories::org_marketplaces::get_latest_sync(&pool, &mkt.id)
            .await
            .ok()
            .flatten();

        let user_access_count = repositories::org_marketplaces::count_users_with_marketplace_access(&pool, &mkt.id)
            .await
            .unwrap_or(0);

        marketplaces_json.push(json!({
            "id": mkt.id,
            "name": mkt.name,
            "description": mkt.description,
            "github_repo_url": mkt.github_repo_url,
            "has_github": mkt.github_repo_url.is_some(),
            "created_at": mkt.created_at.to_rfc3339(),
            "updated_at": mkt.updated_at.to_rfc3339(),
            "plugin_count": plugin_ids.len(),
            "skill_count": skill_count,
            "agent_count": agent_count,
            "mcp_count": mcp_count,
            "hook_count": 0,
            "plugins": plugins_detail,
            "plugin_ids": plugin_ids,
            "roles": roles,
            "departments": dept_assignments,
            "role_count": role_count,
            "department_count": dept_count,
            "user_access_count": user_access_count,
            "last_sync": latest_sync.map(|s| json!({
                "action": s.action,
                "status": s.status,
                "commit_hash": s.commit_hash,
                "plugin_count": s.plugin_count,
                "created_at": s.created_at.to_rfc3339(),
            })),
        }));
    }

    let data = json!({
        "page": "org-marketplaces",
        "title": "Org Marketplaces",
        "marketplaces": marketplaces_json,
        "all_departments": all_dept_names,
        "all_departments_detail": all_departments_detail,
        "all_roles": all_roles_json,
        "stats": {
            "marketplace_count": marketplaces.len(),
            "total_plugins": total_plugins,
            "total_skills": total_skills,
        },
    });

    super::render_page(&engine, "org-marketplaces", &data, &user_ctx, &mkt_ctx)
}
