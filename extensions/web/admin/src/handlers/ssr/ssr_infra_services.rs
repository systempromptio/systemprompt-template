use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub async fn infra_services_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(resp) => return *resp,
    };

    let agents = repositories::list_agents(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents for infra services page");
        vec![]
    });

    let agents_json: Vec<serde_json::Value> = agents
        .iter()
        .map(|a| {
            json!({
                "id": a.id.as_str(),
                "name": a.name,
                "description": a.description,
                "enabled": a.enabled,
                "port": a.port,
                "endpoint": a.endpoint,
                "is_primary": a.is_primary,
                "mcp_server_count": a.mcp_servers.len(),
                "skill_count": a.skills.len(),
            })
        })
        .collect();

    let agent_count = agents_json.len();
    let enabled_count = agents.iter().filter(|a| a.enabled).count();

    let data = json!({
        "page": "infra-services",
        "title": "Infrastructure — Services",
        "cli_command": "systemprompt infra services status --detailed",
        "registry_command": "systemprompt admin agents registry",
        "agents": agents_json,
        "has_agents": !agents.is_empty(),
        "agent_count": agent_count,
        "enabled_count": enabled_count,
    });
    super::render_page(&engine, "infra-services", &data, &user_ctx, &mkt_ctx)
}
