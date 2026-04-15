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

pub async fn analytics_content_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let data = json!({
        "page": "analytics-content",
        "title": "Content & Traffic",
        "cli_command": "systemprompt analytics content stats",
        "window_label": "last 7 days",
        "rows": Vec::<serde_json::Value>::new(),
        "has_rows": false,
    });

    super::render_page(&engine, "analytics-content", &data, &user_ctx, &mkt_ctx)
}
