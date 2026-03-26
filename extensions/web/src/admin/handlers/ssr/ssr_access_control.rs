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

#[allow(clippy::too_many_lines)]
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

    let admin_roles = vec!["admin".to_string()];
    let plugins = repositories::list_plugins_for_roles(&services_path, &admin_roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins");
            vec![]
        });

    let agents = repositories::list_agents(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });

    let mcp_servers = repositories::list_mcp_servers(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list MCP servers");
        vec![]
    });

    let (rules_res, dept_res) = tokio::join!(
        repositories::access_control::list_all_rules(&pool),
        repositories::fetch_department_stats(&pool),
    );

    let all_rules = rules_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch access control rules");
        vec![]
    });

    let departments = dept_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch department stats");
        vec![]
    });

    let mut rules_map: HashMap<
        (String, String),
        Vec<&crate::admin::types::access_control::AccessControlRule>,
    > = HashMap::new();
    for rule in &all_rules {
        rules_map
            .entry((rule.entity_type.clone(), rule.entity_id.clone()))
            .or_default()
            .push(rule);
    }

    let known_roles = vec!["admin", "developer", "analyst", "viewer"];
    let dept_names: Vec<&str> = departments.iter().map(|d| d.department.as_str()).collect();

    let plugins_json: Vec<serde_json::Value> = plugins
        .iter()
        .map(|p| {
            let detail = repositories::get_plugin_detail(&services_path, &p.id);
            let yaml_roles: Vec<String> = detail.ok().flatten().map_or_else(Vec::new, |d| d.roles);

            build_entity_json(
                &p.id,
                &p.name,
                &p.description,
                p.enabled,
                "plugin",
                &rules_map,
                &known_roles,
                &dept_names,
                &departments,
                Some(&yaml_roles),
            )
        })
        .collect();

    let agents_json: Vec<serde_json::Value> = agents
        .iter()
        .map(|a| {
            build_entity_json(
                &a.id,
                &a.name,
                &a.description,
                a.enabled,
                "agent",
                &rules_map,
                &known_roles,
                &dept_names,
                &departments,
                None,
            )
        })
        .collect();

    let mcp_json: Vec<serde_json::Value> = mcp_servers
        .iter()
        .map(|m| {
            build_entity_json(
                &m.id,
                &m.id,
                &m.description,
                m.enabled,
                "mcp_server",
                &rules_map,
                &known_roles,
                &dept_names,
                &departments,
                None,
            )
        })
        .collect();

    let dept_json: Vec<serde_json::Value> = departments
        .iter()
        .map(|d| {
            json!({
                "department": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
            })
        })
        .collect();

    let data = json!({
        "page": "access-control",
        "title": "Access Control",
        "plugins": plugins_json,
        "agents": agents_json,
        "mcp_servers": mcp_json,
        "departments": dept_json,
        "known_roles": known_roles,
        "stats": {
            "plugin_count": plugins.len(),
            "agent_count": agents.len(),
            "mcp_count": mcp_servers.len(),
            "department_count": departments.len(),
        },
    });

    super::render_page(&engine, "access-control", &data, &user_ctx, &mkt_ctx)
}

#[allow(clippy::too_many_arguments)]
fn build_entity_json(
    id: &str,
    name: &str,
    description: &str,
    enabled: bool,
    entity_type: &str,
    rules_map: &HashMap<
        (String, String),
        Vec<&crate::admin::types::access_control::AccessControlRule>,
    >,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[crate::admin::types::DepartmentStats],
    yaml_roles: Option<&[String]>,
) -> serde_json::Value {
    let key = (entity_type.to_string(), id.to_string());
    let entity_rules = rules_map.get(&key);

    let mut role_count = 0;
    let roles: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|role_name| {
            let from_yaml = yaml_roles.is_some_and(|yr| yr.iter().any(|r| r == role_name));
            let from_db = entity_rules.is_some_and(|rules| {
                rules.iter().any(|r| {
                    r.rule_type == "role" && r.rule_value == *role_name && r.access == "allow"
                })
            });
            let assigned = from_yaml || from_db;
            if assigned {
                role_count += 1;
            }
            json!({
                "name": role_name,
                "assigned": assigned,
            })
        })
        .collect();

    let mut dept_count = 0;
    let total_departments = dept_names.len();
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
            let user_count = departments
                .iter()
                .find(|d| d.department == *dept_name)
                .map_or(0, |d| d.user_count);
            if assigned {
                dept_count += 1;
            }
            json!({
                "name": dept_name,
                "assigned": assigned,
                "default_included": default_included,
                "user_count": user_count,
            })
        })
        .collect();

    json!({
        "id": id,
        "name": name,
        "description": description,
        "enabled": enabled,
        "entity_type": entity_type,
        "roles": roles,
        "departments": dept_assignments,
        "role_count": role_count,
        "department_count": dept_count,
        "total_departments": total_departments,
    })
}
