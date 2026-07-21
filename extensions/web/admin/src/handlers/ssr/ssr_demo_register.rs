//! SSR page for demo account registration.

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;

use super::{ACCESS_DENIED_HTML, render_typed_page};

#[derive(Debug, Serialize)]
struct DemoRegisterContext {
    title: &'static str,
    page: &'static str,
}

pub(crate) async fn demo_register_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML.to_owned())).into_response();
    }

    let ctx = DemoRegisterContext {
        title: "Demo User Registration",
        page: "demo-register",
    };

    render_typed_page(&engine, "demo-register", &ctx, &user_ctx, &mkt_ctx)
}
