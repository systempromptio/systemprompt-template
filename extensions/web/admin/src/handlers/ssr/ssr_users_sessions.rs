use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub async fn users_sessions_page(
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

    let current_session = json!({
        "user_id": user_ctx.user_id.to_string(),
        "username": user_ctx.username,
        "email": user_ctx.email.to_string(),
        "is_admin": user_ctx.is_admin,
        "roles": user_ctx.roles,
    });

    let data = json!({
        "page": "users-sessions",
        "title": "User Sessions",
        "cli_command": "systemprompt admin session show",
        "cli_command_list": "systemprompt admin session list",
        "current_session": current_session,
        "profiles": [],
    });
    super::render_page(&engine, "users-sessions", &data, &user_ctx, &mkt_ctx)
}
