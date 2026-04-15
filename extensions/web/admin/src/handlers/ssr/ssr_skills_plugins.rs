use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

/// Read-only dashboard surface for the `core plugins` / `core hooks` /
/// `plugins` CLI families.
///
/// Mirrors `demo/skills/04-plugin-management.sh`: plugin listing with
/// validation, hook inventory, and extension capabilities. CRUD lives on
/// `/admin/plugins`; this page is strictly read-only.
pub async fn skills_plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
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
        Err(r) => return *r,
    };

    let plugins = repositories::list_plugins_for_roles_full(&services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for roles");
            vec![]
        });

    let plugin_rows: Vec<serde_json::Value> = plugins
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "enabled": p.enabled,
                "skill_count": p.skills.len(),
                "agent_count": p.agents.len(),
                "mcp_count": p.mcp_servers.len(),
                "hook_count": p.hooks.len(),
            })
        })
        .collect();

    let total_hooks: usize = plugins.iter().map(|p| p.hooks.len()).sum();
    let total_skills: usize = plugins.iter().map(|p| p.skills.len()).sum();
    let enabled_count = plugins.iter().filter(|p| p.enabled).count();

    let data = json!({
        "page": "skills-plugins",
        "title": "Plugin Inspection",
        "plugins": plugin_rows,
        "stats": {
            "plugin_count": plugins.len(),
            "plugin_enabled": enabled_count,
            "hook_count": total_hooks,
            "skill_count": total_skills,
        },
        "cli_commands": [
            { "label": "List plugins",          "cmd": "systemprompt core plugins list" },
            { "label": "Show plugin",           "cmd": "systemprompt core plugins show enterprise-demo" },
            { "label": "Validate plugin",       "cmd": "systemprompt core plugins validate enterprise-demo" },
            { "label": "List hooks",            "cmd": "systemprompt core hooks list" },
            { "label": "Validate hooks",        "cmd": "systemprompt core hooks validate" },
            { "label": "List extensions",       "cmd": "systemprompt plugins list" },
            { "label": "Extension capabilities","cmd": "systemprompt plugins capabilities" },
        ],
    });
    super::render_page(&engine, "skills-plugins", &data, &user_ctx, &mkt_ctx)
}
