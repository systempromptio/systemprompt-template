use axum::{
    extract::Extension,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;

use crate::repositories::external_agents_grp::{list_external_agents, ExternalAgentRow};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

fn row_to_json(r: &ExternalAgentRow) -> serde_json::Value {
    json!({
        "id": r.id,
        "display_name": r.display_name,
        "kind": r.kind,
        "kind_label": match r.kind.as_str() {
            "desktop_app" => "Desktop app",
            "cli_tool" => "CLI tool",
            other => other,
        },
        "description": r.description,
        "platforms": r.platforms,
        "docs_url": r.docs_url,
    })
}

pub async fn external_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let rows = list_external_agents();
    let (enabled, disabled): (Vec<_>, Vec<_>) = rows.iter().partition(|r| r.enabled);
    let enabled_json: Vec<_> = enabled.iter().map(|r| row_to_json(r)).collect();
    let disabled_json: Vec<_> = disabled.iter().map(|r| row_to_json(r)).collect();

    let data = json!({
        "page": "external-agents",
        "title": "External Agents (Super-Agents)",
        "subtitle": "Native apps and CLI tools that run off our infrastructure and connect to the gateway via the systemprompt-bridge. Distinct from A2A agents under /admin/agents.",
        "yaml_dir": "services/external_agents/",
        "enabled": enabled_json,
        "disabled": disabled_json,
        "has_enabled": !enabled.is_empty(),
        "has_disabled": !disabled.is_empty(),
        "has_any": !rows.is_empty(),
    });

    super::render_page(&engine, "external-agents", &data, &user_ctx, &mkt_ctx)
}
