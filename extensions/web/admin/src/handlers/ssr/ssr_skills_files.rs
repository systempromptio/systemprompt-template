use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

/// Read-only dashboard surface for the `core files` CLI family.
///
/// Mirrors `demo/skills/03-file-management.sh`: file listing, upload
/// configuration, and storage statistics. The underlying commands are
/// CLI-only, so the page renders an instructional empty state pointing at
/// the CLI.
pub async fn skills_files_page(
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
        "page": "skills-files",
        "title": "File Management",
        "cli_commands": [
            { "label": "List files",    "cmd": "systemprompt core files list" },
            { "label": "Upload config", "cmd": "systemprompt core files config" },
            { "label": "File stats",    "cmd": "systemprompt core files stats" },
        ],
    });
    super::render_page(&engine, "skills-files", &data, &user_ctx, &mkt_ctx)
}
