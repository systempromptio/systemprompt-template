use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

/// Read-only dashboard surface for the `core content` CLI family.
///
/// Mirrors `demo/skills/02-content-management.sh`: content listing, search,
/// popularity, and health. The underlying commands are CLI-only (no admin
/// repository) so the page renders an instructional empty state that points
/// operators at the CLI.
pub async fn skills_content_page(
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

    let data = json!({
        "page": "skills-content",
        "title": "Content Management",
        "cli_commands": [
            { "label": "List content",    "cmd": "systemprompt core content list" },
            { "label": "Search content",  "cmd": "systemprompt core content search \"governance\"" },
            { "label": "Popular content", "cmd": "systemprompt core content popular documentation" },
            { "label": "Content health",  "cmd": "systemprompt core content status --source documentation" },
        ],
    });
    super::render_page(&engine, "skills-content", &data, &user_ctx, &mkt_ctx)
}
