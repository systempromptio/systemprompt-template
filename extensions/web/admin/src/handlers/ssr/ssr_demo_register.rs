use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde_json::json;

use super::{ACCESS_DENIED_HTML, render_page};

pub(crate) async fn demo_register_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML.to_owned())).into_response();
    }

    let data = json!({
        "title": "Demo User Registration",
        "page": "demo-register",
    });

    render_page(&engine, "demo-register", &data, &user_ctx, &mkt_ctx)
}
