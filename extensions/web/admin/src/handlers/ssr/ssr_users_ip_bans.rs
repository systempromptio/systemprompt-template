use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::Extension,
    response::{IntoResponse, Response},
};
use serde_json::json;

pub async fn users_ip_bans_page(
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

    let data = json!({
        "page": "users-ip-bans",
        "title": "IP Bans",
        "cli_command": "systemprompt admin users ban list",
        "cli_command_add": "systemprompt admin users ban add <ip> --reason \"<reason>\"",
        "cli_command_remove": "systemprompt admin users ban remove <ip>",
        "bans": [],
    });
    super::render_page(&engine, "users-ip-bans", &data, &user_ctx, &mkt_ctx)
}
