use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

pub async fn governance_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let hooks = super::ssr_hooks::list_hooks_from_filesystem(&services_path);

    let hook_rows: Vec<serde_json::Value> = hooks
        .iter()
        .map(|h| {
            json!({
                "id": h.id.as_str(),
                "plugin_id": h.plugin_id,
                "name": h.name,
                "event": h.event,
                "matcher": h.matcher,
                "is_async": h.is_async,
                "enabled": h.enabled,
                "system": h.system,
                "valid": !h.command.is_empty() && !h.event.is_empty(),
            })
        })
        .collect();

    let total_hooks = hook_rows.len();
    let system_hooks = hooks.iter().filter(|h| h.system).count();
    let enabled_hooks = hooks.iter().filter(|h| h.enabled).count();

    let data = json!({
        "page": "governance-hooks",
        "title": "Governance Hooks",
        "hero_title": "Governance Hooks",
        "hero_subtitle": "Hook list with config and validation status (read-only governance view)",
        "cli_command": "systemprompt core hooks list",
        "cli_validate": "systemprompt core hooks validate",
        "total": total_hooks,
        "system": system_hooks,
        "enabled": enabled_hooks,
        "has_hooks": !hook_rows.is_empty(),
        "hooks": hook_rows,
    });

    super::render_page(&engine, "governance-hooks", &data, &user_ctx, &mkt_ctx)
}
