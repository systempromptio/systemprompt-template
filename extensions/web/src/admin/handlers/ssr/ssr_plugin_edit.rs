use std::collections::HashMap;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, PluginDetail, UserContext};
use axum::{
    extract::{Extension, Query},
    response::Response,
};
use serde_json::json;

pub(crate) async fn plugin_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let plugin_id = params.get("id");
    let is_edit = plugin_id.is_some();
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let plugin: Option<PluginDetail> = if let Some(id) = plugin_id {
        repositories::get_plugin_detail(&services_path, id)
            .map_err(|e| {
                tracing::warn!(error = %e, plugin_id = %id, "Failed to fetch plugin detail");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let all_skills = repositories::list_all_skill_ids(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list skill IDs");
        vec![]
    });
    let all_agents = repositories::list_agents(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });
    let all_mcp = repositories::list_mcp_servers(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list MCP servers");
        vec![]
    });

    let known_roles = ["admin", "developer", "analyst", "viewer"];
    let plugin_roles: Vec<&str> = plugin
        .as_ref()
        .map_or(vec![], |p| p.roles.iter().map(String::as_str).collect());
    let plugin_skills: Vec<&str> = plugin
        .as_ref()
        .map_or(vec![], |p| p.skills.iter().map(String::as_str).collect());
    let plugin_agents: Vec<&str> = plugin
        .as_ref()
        .map_or(vec![], |p| p.agents.iter().map(String::as_str).collect());
    let plugin_mcp: Vec<&str> = plugin.as_ref().map_or(vec![], |p| {
        p.mcp_servers.iter().map(String::as_str).collect()
    });

    let roles_list: Vec<serde_json::Value> = known_roles
        .iter()
        .map(|r| json!({ "value": r, "checked": plugin_roles.contains(r) }))
        .collect();
    let skills_list: Vec<serde_json::Value> = all_skills
        .iter()
        .map(|s| json!({ "value": s, "checked": plugin_skills.contains(&s.as_str()) }))
        .collect();
    let agents_list: Vec<serde_json::Value> = all_agents
        .iter()
        .map(|a| json!({ "value": a.id, "name": a.name, "checked": plugin_agents.contains(&a.id.as_str()) }))
        .collect();
    let mcp_list: Vec<serde_json::Value> = all_mcp
        .iter()
        .map(|m| json!({ "value": m.id, "name": m.id, "checked": plugin_mcp.contains(&m.id.as_str()) }))
        .collect();

    let keywords_csv = plugin
        .as_ref()
        .map_or(String::new(), |p| p.keywords.join(", "));

    let data = json!({
        "page": "plugin-edit",
        "title": if is_edit { "Edit Plugin" } else { "Create Plugin" },
        "is_edit": is_edit,
        "plugin": plugin,
        "keywords_csv": keywords_csv,
        "roles_list": roles_list,
        "skills_list": skills_list,
        "agents_list": agents_list,
        "mcp_list": mcp_list,
    });
    super::render_page(&engine, "plugin-edit", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn plugin_create_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let all_skills = repositories::list_all_skill_ids(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list skill IDs");
        vec![]
    });
    let all_agents = repositories::list_agents(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });
    let all_mcp = repositories::list_mcp_servers(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list MCP servers");
        vec![]
    });

    let known_roles = ["admin", "developer", "analyst", "viewer"];
    let roles_list: Vec<serde_json::Value> =
        known_roles.iter().map(|r| json!({ "value": r })).collect();
    let skills_list: Vec<serde_json::Value> = all_skills
        .iter()
        .map(|s| json!({ "value": s, "name": s }))
        .collect();
    let agents_list: Vec<serde_json::Value> = all_agents
        .iter()
        .map(|a| json!({ "value": a.id, "name": a.name }))
        .collect();
    let mcp_list: Vec<serde_json::Value> = all_mcp
        .iter()
        .map(|m| json!({ "value": m.id, "name": m.id }))
        .collect();

    let data = json!({
        "page": "plugin-create",
        "title": "Create Plugin",
        "roles_list": roles_list,
        "skills_list": skills_list,
        "agents_list": agents_list,
        "mcp_list": mcp_list,
        "hook_events": ["PostToolUse", "SessionStart", "PreToolUse", "Notification"],
    });
    super::render_page(&engine, "plugin-create", &data, &user_ctx, &mkt_ctx)
}
