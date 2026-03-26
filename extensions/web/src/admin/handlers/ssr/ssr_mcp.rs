use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

#[allow(clippy::too_many_lines)]
pub(crate) async fn mcp_servers_page(
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
        repositories::list_mcp_servers(&services_path).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list MCP servers");
            vec![]
        })
    } else {
        let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to list plugins for roles");
                vec![]
            });
        let visible_mcp_ids: std::collections::HashSet<String> = plugins
            .iter()
            .flat_map(|p| p.mcp_servers.iter().cloned())
            .collect();
        repositories::list_mcp_servers(&services_path)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to list MCP servers");
                vec![]
            })
            .into_iter()
            .filter(|m| visible_mcp_ids.contains(&m.id))
            .collect()
    };

    let (_skill_map, _agent_map, mcp_plugin_map) =
        repositories::build_entity_plugin_maps(&services_path);

    let (rules_res, dept_res, service_status_res, tool_stats_res) = tokio::join!(
        repositories::access_control::list_all_rules(&pool),
        repositories::fetch_department_stats(&pool),
        sqlx::query_as::<_, (String, String, Option<i32>)>(
            "SELECT name, status, pid FROM services WHERE module_name = 'mcp'"
        )
        .fetch_all(pool.as_ref()),
        sqlx::query_as::<_, (String, i64, i64)>(
            "SELECT server_name, COUNT(*) as total, COUNT(*) FILTER (WHERE status = 'success') as success_count FROM mcp_tool_executions GROUP BY server_name"
        )
        .fetch_all(pool.as_ref()),
    );

    let all_rules = rules_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch access control rules");
        vec![]
    });

    let departments = dept_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch department stats");
        vec![]
    });

    let service_status_map: HashMap<String, (String, Option<i32>)> = service_status_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch service status");
            vec![]
        })
        .into_iter()
        .map(|(name, status, pid)| (name, (status, pid)))
        .collect();

    let tool_stats_map: HashMap<String, (i64, i64)> = tool_stats_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch tool stats");
            vec![]
        })
        .into_iter()
        .map(|(name, total, success)| (name, (total, success)))
        .collect();

    let mut rules_map: HashMap<
        String,
        Vec<&crate::admin::types::access_control::AccessControlRule>,
    > = HashMap::new();
    for rule in &all_rules {
        if rule.entity_type == "mcp_server" {
            rules_map
                .entry(rule.entity_id.clone())
                .or_default()
                .push(rule);
        }
    }

    let known_roles = ["admin", "developer", "analyst", "viewer"];

    let mcp_json: Vec<serde_json::Value> = mcp_servers
        .iter()
        .map(|m| {
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

            let roles: Vec<serde_json::Value> = known_roles
                .iter()
                .filter_map(|role_name| {
                    let assigned = entity_rules.is_some_and(|rules| {
                        rules.iter().any(|r| {
                            r.rule_type == "role"
                                && r.rule_value == *role_name
                                && r.access == "allow"
                        })
                    });
                    if assigned {
                        Some(json!({"name": role_name, "assigned": true}))
                    } else {
                        None
                    }
                })
                .collect();

            let dept_badges: Vec<serde_json::Value> = departments
                .iter()
                .filter_map(|d| {
                    let rule = entity_rules.and_then(|rules| {
                        rules
                            .iter()
                            .find(|r| r.rule_type == "department" && r.rule_value == d.department)
                    });
                    let assigned = rule.is_some_and(|r| r.access == "allow");
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
                .collect();

            let is_internal = m.server_type == "internal";

            let (service_status, service_pid) = if is_internal {
                service_status_map
                    .get(&m.id)
                    .map_or(("unknown", None), |(s, p)| (s.as_str(), *p))
            } else {
                ("", None)
            };

            #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
            let (total_executions, success_rate) = if is_internal {
                tool_stats_map
                    .get(&m.id)
                    .map_or((0, 0), |(total, success)| {
                        let rate = if *total > 0 {
                            (*success as f64 / *total as f64 * 100.0).round() as i64
                        } else {
                            0
                        };
                        (*total, rate)
                    })
            } else {
                (0, 0)
            };

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
                "service_status": service_status,
                "service_pid": service_pid,
                "total_executions": total_executions,
                "success_rate": success_rate,
            })
        })
        .collect();

    let data = json!({
        "page": "mcp-servers",
        "title": "MCP Servers",
        "mcp_servers": mcp_json,
    });
    super::render_page(&engine, "mcp-servers", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn mcp_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let server_id = params.get("id");
    let is_edit = server_id.is_some();
    let server = if let Some(id) = server_id {
        let services_path = match super::get_services_path() {
            Ok(p) => p,
            Err(r) => return *r,
        };
        repositories::get_mcp_server(&services_path, id)
            .map_err(|e| {
                tracing::warn!(error = %e, server_id = %id, "Failed to fetch MCP server");
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
        repositories::get_mcp_server_raw_yaml(&sp, id)
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
