//! SSR page for connecting a client — Claude Code, Claude Desktop or `OpenCode`
//! — to this gateway with a single-use connect code.

use axum::extract::Extension;
use axum::response::Response;
use serde::Serialize;

use crate::error::AdminHtmlResult;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::services::bridge_profile;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Debug, Serialize)]
struct ConnectPageView {
    page: &'static str,
    title: &'static str,
    user_email: String,
    gateway: Option<String>,
    // Why: whether a gateway is configured, and so whether the page may offer
    // to issue a connect code. The code itself is never part of page data.
    bridge_connect_available: bool,
    breadcrumbs: Vec<BreadcrumbView>,
}

pub(crate) async fn connect_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    let (_, gateway) = bridge_profile::read_config_strings();
    let view = ConnectPageView {
        page: "connect",
        title: "Connect a client",
        user_email: user_ctx.email.as_str().to_owned(),
        bridge_connect_available: gateway.is_some(),
        gateway,
        breadcrumbs: vec![
            BreadcrumbView::link("Account", "/admin/profile"),
            BreadcrumbView::current("Connect"),
        ],
    };
    Ok(render_typed_page(
        &engine, "connect", &view, &user_ctx, &mkt_ctx,
    ))
}
