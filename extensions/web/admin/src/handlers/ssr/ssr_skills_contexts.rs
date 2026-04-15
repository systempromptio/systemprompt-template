use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

/// Read-only dashboard surface for the `core contexts` CLI family.
///
/// Mirrors `demo/skills/05-contexts.sh`: list, create, show, edit, delete
/// conversation contexts. Contexts are owned by the core CLI and not
/// surfaced through an admin repository, so the page renders an
/// instructional empty state pointing at the CLI.
pub async fn skills_contexts_page(
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
        "page": "skills-contexts",
        "title": "Conversation Contexts",
        "cli_commands": [
            { "label": "List contexts",   "cmd": "systemprompt core contexts list" },
            { "label": "Create context",  "cmd": "systemprompt core contexts create --name \"My Context\"" },
            { "label": "Show context",    "cmd": "systemprompt core contexts show <context-id>" },
            { "label": "Rename context",  "cmd": "systemprompt core contexts edit <context-id> --name \"New Name\"" },
            { "label": "Delete context",  "cmd": "systemprompt core contexts delete <context-id>" },
        ],
    });
    super::render_page(&engine, "skills-contexts", &data, &user_ctx, &mkt_ctx)
}
