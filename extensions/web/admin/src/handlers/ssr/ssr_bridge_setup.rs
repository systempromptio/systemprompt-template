//! SSR page walking a user through bridge installation.

use axum::extract::Extension;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Serialize;

use crate::error::AdminHtmlResult;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_helpers::render_typed_page;

#[derive(Debug, Serialize)]
struct SetupPageData {
    gateway_url: String,
    user_email: String,
    download_base_url: Option<String>,
    install_command: Option<String>,
}

pub(crate) async fn bridge_setup_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    headers: HeaderMap,
) -> AdminHtmlResult<Response> {
    let gateway_url = derive_gateway_url(&headers);
    let download_base_url = crate::services::bridge_downloads::download_base(&gateway_url);
    let install_command =
        crate::services::bridge_downloads::install_command(&gateway_url, None, "claude-code");
    let data = SetupPageData {
        install_command,
        gateway_url,
        user_email: user_ctx.email.to_string(),
        download_base_url,
    };
    Ok(render_typed_page(
        &engine,
        "bridge-setup",
        &data,
        &user_ctx,
        &mkt_ctx,
    ))
}

fn derive_gateway_url(headers: &HeaderMap) -> String {
    let scheme = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("http");
    let host = headers
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:8080");
    format!("{scheme}://{host}")
}
