//! SSR page for demo account registration.

use crate::error::{AdminError, AdminHtmlResult};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::response::Response;
use serde::Serialize;

use super::render_typed_page;

#[derive(Debug, Serialize)]
struct DemoRegisterContext {
    title: &'static str,
    page: &'static str,
}

pub(crate) async fn demo_register_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }

    let ctx = DemoRegisterContext {
        title: "Demo User Registration",
        page: "demo-register",
    };

    Ok(render_typed_page(
        &engine,
        "demo-register",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}
