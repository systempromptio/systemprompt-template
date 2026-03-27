use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
use serde_json::json;

use super::{render_page, ACCESS_DENIED_HTML};

pub(crate) async fn demo_register_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML.to_string())).into_response();
    }

    let data = json!({
        "title": "Demo User Registration",
        "page": "demo-register",
    });

    render_page(&engine, "demo-register", &data, &user_ctx, &mkt_ctx)
}
