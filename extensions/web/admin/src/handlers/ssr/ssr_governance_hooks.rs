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

    let hook_rows: Vec<serde_json::Value> = Vec::new();
    let total_hooks = 0usize;
    let system_hooks = 0usize;
    let enabled_hooks = 0usize;

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
