use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query},
    response::Response,
};
use serde_json::json;

/// Agent configuration detail page — validation, MCP tool access, availability.
///
/// Mirrors `demo/agents/02-agent-config.sh`:
///   - `admin agents validate <id>` → config validity
///   - `admin agents tools <id>`    → MCP servers assigned to the agent
///   - `admin agents status`        → endpoint/port availability
pub async fn agent_config_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let agent_id_param = params.get("id").cloned().unwrap_or_default();
    let agent = if agent_id_param.is_empty() {
        None
    } else {
        repositories::find_agent(&services_path, &agent_id_param)
            .map_err(|e| {
                tracing::error!(error = %e, agent_id = %agent_id_param, "Failed to fetch agent");
            })
            .ok()
            .flatten()
    };

    let mut validation_issues: Vec<&str> = Vec::new();
    if let Some(a) = &agent {
        if a.system_prompt.trim().is_empty() {
            validation_issues.push("Missing system prompt");
        }
        if a.name.trim().is_empty() {
            validation_issues.push("Missing display name");
        }
        if a.endpoint.is_none() {
            validation_issues.push("No endpoint configured");
        }
    }
    let is_valid = agent.is_some() && validation_issues.is_empty();

    let mcp_servers_json: Vec<serde_json::Value> = agent
        .as_ref()
        .map(|a| {
            a.mcp_servers
                .iter()
                .map(|id| json!({ "id": id.as_str() }))
                .collect()
        })
        .unwrap_or_default();

    let skills_json: Vec<serde_json::Value> = agent
        .as_ref()
        .map(|a| {
            a.skills
                .iter()
                .map(|s| {
                    json!({
                        "id": s.id.as_str(),
                        "name": s.name,
                        "description": s.description,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let cli_command = if agent_id_param.is_empty() {
        "systemprompt admin agents validate <id>".to_string()
    } else {
        format!("systemprompt admin agents validate {agent_id_param}")
    };

    let data = json!({
        "page": "agent-config",
        "title": "Agent Config",
        "active_tab": "config",
        "agent_id": agent_id_param,
        "has_agent": agent.is_some(),
        "agent": agent,
        "is_valid": is_valid,
        "validation_issues": validation_issues,
        "mcp_servers": mcp_servers_json,
        "has_mcp_servers": !mcp_servers_json.is_empty(),
        "skills": skills_json,
        "has_skills": !skills_json.is_empty(),
        "cli_command": cli_command,
    });
    super::render_page(&engine, "agent-config", &data, &user_ctx, &mkt_ctx)
}
