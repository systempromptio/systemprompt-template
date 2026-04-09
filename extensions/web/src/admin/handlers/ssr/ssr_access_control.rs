use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::access_control::AccessControlRule;
use crate::admin::types::{DepartmentStats, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

struct EntityJsonParams<'a> {
    id: &'a str,
    name: &'a str,
    description: &'a str,
    enabled: bool,
    entity_type: &'a str,
    yaml_roles: Option<&'a [String]>,
}

struct AccessControlData {
    all_rules: Vec<AccessControlRule>,
    departments: Vec<DepartmentStats>,
}

fn load_filesystem_entities(
    services_path: &std::path::Path,
) -> (
    Vec<crate::admin::types::PluginOverview>,
    Vec<crate::admin::types::AgentDetail>,
    Vec<crate::admin::types::McpServerDetail>,
) {
    let admin_roles = vec!["admin".to_string()];
    let plugins =
        repositories::list_plugins_for_roles(services_path, &admin_roles).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins");
            vec![]
        });

    let agents = repositories::list_agents(services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });

    let mcp_servers =
        repositories::mcp_servers::list_mcp_servers(services_path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list MCP servers");
            vec![]
        });

    (plugins, agents, mcp_servers)
}

async fn load_access_control_data(pool: &PgPool) -> AccessControlData {
    let (rules_res, dept_res) = tokio::join!(
        repositories::access_control::list_all_rules(pool),
        repositories::fetch_department_stats(pool),
    );

    let all_rules = rules_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch access control rules");
        vec![]
    });

    let departments = dept_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch department stats");
        vec![]
    });

    AccessControlData {
        all_rules,
        departments,
    }
}

fn build_rules_map(
    all_rules: &[AccessControlRule],
) -> HashMap<(String, String), Vec<&AccessControlRule>> {
    let mut rules_map: HashMap<(String, String), Vec<&AccessControlRule>> = HashMap::new();
    for rule in all_rules {
        rules_map
            .entry((rule.entity_type.clone(), rule.entity_id.clone()))
            .or_default()
            .push(rule);
    }
    rules_map
}

fn build_departments_json(departments: &[DepartmentStats]) -> Vec<serde_json::Value> {
    departments
        .iter()
        .map(|d| {
            json!({
                "department": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
            })
        })
        .collect()
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

    let (plugins, agents, mcp_servers) = load_filesystem_entities(&services_path);
    let ac_data = load_access_control_data(&pool).await;
    let data = assemble_page_data(&services_path, &plugins, &agents, &mcp_servers, &ac_data);

    super::render_page(&engine, "access-control", &data, &user_ctx, &mkt_ctx)
}

fn assemble_page_data(
    services_path: &std::path::Path,
    plugins: &[crate::admin::types::PluginOverview],
    agents: &[crate::admin::types::AgentDetail],
    mcp_servers: &[crate::admin::types::McpServerDetail],
    ac_data: &AccessControlData,
) -> serde_json::Value {
    let rules_map = build_rules_map(&ac_data.all_rules);
    let known_roles = vec!["admin", "developer", "analyst", "viewer"];
    let dept_names: Vec<&str> = ac_data
        .departments
        .iter()
        .map(|d| d.department.as_str())
        .collect();

    let plugins_json = build_plugins_json(
        services_path,
        plugins,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );
    let agents_json = build_agents_json(
        agents,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );
    let mcp_json = build_mcp_json(
        mcp_servers,
        &rules_map,
        &known_roles,
        &dept_names,
        &ac_data.departments,
    );

    json!({
        "page": "access-control",
        "title": "Access Control",
        "plugins": plugins_json,
        "agents": agents_json,
        "mcp_servers": mcp_json,
        "departments": build_departments_json(&ac_data.departments),
        "known_roles": known_roles,
        "stats": {
            "plugin_count": plugins.len(),
            "agent_count": agents.len(),
            "mcp_count": mcp_servers.len(),
            "department_count": ac_data.departments.len(),
        },
    })
}

fn build_plugins_json(
    services_path: &std::path::Path,
    plugins: &[crate::admin::types::PluginOverview],
    rules_map: &HashMap<(String, String), Vec<&AccessControlRule>>,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> Vec<serde_json::Value> {
    plugins
        .iter()
        .map(|p| {
            let detail = repositories::find_plugin_detail(services_path, &p.id);
            let yaml_roles: Vec<String> = detail.ok().flatten().map_or_else(Vec::new, |d| d.roles);
            let params = EntityJsonParams {
                id: &p.id,
                name: &p.name,
                description: &p.description,
                enabled: p.enabled,
                entity_type: "plugin",
                yaml_roles: Some(&yaml_roles),
            };
            build_entity_json(&params, rules_map, known_roles, dept_names, departments)
        })
        .collect()
}

fn build_agents_json(
    agents: &[crate::admin::types::AgentDetail],
    rules_map: &HashMap<(String, String), Vec<&AccessControlRule>>,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> Vec<serde_json::Value> {
    agents
        .iter()
        .map(|a| {
            let params = EntityJsonParams {
                id: &a.id,
                name: &a.name,
                description: &a.description,
                enabled: a.enabled,
                entity_type: "agent",
                yaml_roles: None,
            };
            build_entity_json(&params, rules_map, known_roles, dept_names, departments)
        })
        .collect()
}

fn build_mcp_json(
    mcp_servers: &[crate::admin::types::McpServerDetail],
    rules_map: &HashMap<(String, String), Vec<&AccessControlRule>>,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> Vec<serde_json::Value> {
    mcp_servers
        .iter()
        .map(|m| {
            let params = EntityJsonParams {
                id: &m.id,
                name: &m.id,
                description: &m.description,
                enabled: m.enabled,
                entity_type: "mcp_server",
                yaml_roles: None,
            };
            build_entity_json(&params, rules_map, known_roles, dept_names, departments)
        })
        .collect()
}

fn build_entity_json(
    params: &EntityJsonParams<'_>,
    rules_map: &HashMap<(String, String), Vec<&AccessControlRule>>,
    known_roles: &[&str],
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> serde_json::Value {
    let key = (params.entity_type.to_string(), params.id.to_string());
    let entity_rules = rules_map.get(&key);

    let (roles, role_count) = build_roles_json(entity_rules, known_roles, params.yaml_roles);
    let (dept_assignments, dept_count) = build_dept_json(entity_rules, dept_names, departments);

    json!({
        "id": params.id,
        "name": params.name,
        "description": params.description,
        "enabled": params.enabled,
        "entity_type": params.entity_type,
        "roles": roles,
        "departments": dept_assignments,
        "role_count": role_count,
        "department_count": dept_count,
        "total_departments": dept_names.len(),
    })
}

fn build_roles_json(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    known_roles: &[&str],
    yaml_roles: Option<&[String]>,
) -> (Vec<serde_json::Value>, usize) {
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
    (roles, role_count)
}

fn build_dept_json(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    dept_names: &[&str],
    departments: &[DepartmentStats],
) -> (Vec<serde_json::Value>, usize) {
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
    (dept_assignments, dept_count)
}
