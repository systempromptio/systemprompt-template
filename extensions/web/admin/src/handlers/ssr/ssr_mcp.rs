use std::collections::HashMap;
use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::access_control::{AccessControlRule, AccessDecision, RuleType};
use crate::types::{DepartmentStats, MarketplaceContext, McpServerDetail, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

fn build_mcp_rules_map(
    all_rules: &[AccessControlRule],
) -> HashMap<String, Vec<&AccessControlRule>> {
    let mut rules_map: HashMap<String, Vec<&AccessControlRule>> = HashMap::new();
    for rule in all_rules {
        if rule.entity_type == "mcp_server" {
            rules_map
                .entry(rule.entity_id.clone())
                .or_default()
                .push(rule);
        }
    }
    rules_map
}

fn build_role_badges(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    known_roles: &[&str],
) -> Vec<serde_json::Value> {
    known_roles
        .iter()
        .filter_map(|role_name| {
            let assigned = entity_rules.is_some_and(|rules| {
                rules.iter().any(|r| {
                    r.rule_type == RuleType::Role && r.rule_value == *role_name && r.access == AccessDecision::Allow
                })
            });
            if assigned {
                Some(json!({"name": role_name, "assigned": true}))
            } else {
                None
            }
        })
        .collect()
}

fn build_dept_badges(
    entity_rules: Option<&Vec<&AccessControlRule>>,
    departments: &[DepartmentStats],
) -> Vec<serde_json::Value> {
    departments
        .iter()
        .filter_map(|d| {
            let rule = entity_rules.and_then(|rules| {
                rules
                    .iter()
                    .find(|r| r.rule_type == RuleType::Department && r.rule_value == d.department)
            });
            let assigned = rule.is_some_and(|r| r.access == AccessDecision::Allow);
            if assigned {
                let default_included = rule.is_some_and(|r| r.default_included);
                Some(json!({
                    "name": d.department,
                    "assigned": true,
                    "default_included": default_included,
                }))
            } else {
                None
            }
        })
        .collect()
}

fn build_mcp_server_json(
    m: &McpServerDetail,
    mcp_plugin_map: &HashMap<String, Vec<(String, String)>>,
    rules_map: &HashMap<String, Vec<&AccessControlRule>>,
    departments: &[DepartmentStats],
    known_roles: &[&str],
) -> serde_json::Value {
    let assigned_plugins: Vec<serde_json::Value> = mcp_plugin_map
        .get(&m.id)
        .map(|plugins| {
            plugins
                .iter()
                .map(|(pid, pname)| json!({"id": pid, "name": pname}))
                .collect()
        })
        .unwrap_or_default();

    let entity_rules = rules_map.get(&m.id);
    let roles = build_role_badges(entity_rules, known_roles);
    let dept_badges = build_dept_badges(entity_rules, departments);
    let is_internal = m.server_type == "internal";

    json!({
        "id": m.id,
        "server_type": m.server_type,
        "is_internal": is_internal,
        "is_external": !is_internal,
        "binary": m.binary,
        "package_name": m.package_name,
        "port": m.port,
        "endpoint": m.endpoint,
        "description": m.description,
        "enabled": m.enabled,
        "oauth_required": m.oauth_required,
        "oauth_scopes": m.oauth_scopes,
        "oauth_audience": m.oauth_audience,
        "assigned_plugins": assigned_plugins,
        "roles": roles,
        "departments": dept_badges,
    })
}

pub async fn mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let mcp_servers = if user_ctx.is_admin {
        repositories::mcp_servers::list_mcp_servers(&services_path).unwrap_or_else(|e| {
            tracing::error!(error = %e, "Failed to list MCP servers");
            vec![]
        })
    } else {
        let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to list plugins for roles");
                vec![]
            });
        let visible_mcp_ids: std::collections::HashSet<String> = plugins
            .iter()
            .flat_map(|p| p.mcp_servers.iter().cloned())
            .collect();
        repositories::mcp_servers::list_mcp_servers(&services_path)
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, "Failed to list MCP servers");
                vec![]
            })
            .into_iter()
            .filter(|m| visible_mcp_ids.contains(&m.id))
            .collect()
    };

    let (_skill_map, _agent_map, mcp_plugin_map) =
        repositories::build_entity_plugin_maps(&services_path);

    let (rules_res, dept_res) = tokio::join!(
        repositories::access_control::list_all_rules(&pool),
        repositories::fetch_department_stats(&pool),
    );

    let all_rules = rules_res.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch access control rules");
        vec![]
    });

    let departments = dept_res.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to fetch department stats");
        vec![]
    });

    let rules_map = build_mcp_rules_map(&all_rules);
    let known_roles = ["admin", "developer", "analyst", "viewer"];

    let mcp_json: Vec<serde_json::Value> = mcp_servers
        .iter()
        .map(|m| build_mcp_server_json(m, &mcp_plugin_map, &rules_map, &departments, &known_roles))
        .collect();

    let data = json!({
        "page": "mcp-servers",
        "title": "MCP Servers",
        "mcp_servers": mcp_json,
    });
    super::render_page(&engine, "mcp-servers", &data, &user_ctx, &mkt_ctx)
}

pub async fn mcp_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let server_id = params.get("id");
    let is_edit = server_id.is_some();
    let server = if let Some(id) = server_id {
        let services_path = match super::get_services_path() {
            Ok(p) => p,
            Err(r) => return *r,
        };
        repositories::mcp_servers::find_mcp_server(&services_path, id)
            .map_err(|e| {
                tracing::error!(error = %e, server_id = %id, "Failed to fetch MCP server");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let is_internal = server.as_ref().is_some_and(|s| s.server_type == "internal");
    let is_external = server.as_ref().is_none_or(|s| s.server_type == "external");
    let raw_yaml = if let Some(id) = server_id {
        let sp = match super::get_services_path() {
            Ok(p) => p,
            Err(r) => return *r,
        };
        repositories::mcp_servers::find_mcp_server_raw_yaml(&sp, id)
            .ok()
            .flatten()
    } else {
        None
    };
    let data = json!({
        "page": "mcp-edit",
        "title": if is_edit { "Edit MCP Server" } else { "Add External MCP Server" },
        "is_edit": is_edit,
        "is_internal": is_internal,
        "is_external": is_external,
        "server": server,
        "raw_yaml": raw_yaml.as_ref().map(|(y, _)| y),
        "yaml_file_name": raw_yaml.as_ref().map(|(_, f)| f),
    });
    super::render_page(&engine, "mcp-edit", &data, &user_ctx, &mkt_ctx)
}
