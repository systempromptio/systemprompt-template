use axum::extract::Extension;
use axum::http::HeaderMap;
use axum::response::Response;
use serde::Serialize;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ssr_helpers::render_typed_page;

const DOWNLOAD_BASE_URL: &str =
    "https://github.com/Ejb503/systemprompt-core/releases/latest/download";

#[derive(Debug, Serialize)]
struct SetupPageData {
    gateway_url: String,
    user_email: String,
    download_base_url: &'static str,
}

pub async fn cowork_setup_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    headers: HeaderMap,
) -> Response {
    let data = SetupPageData {
        gateway_url: derive_gateway_url(&headers),
        user_email: user_ctx.email.to_string(),
        download_base_url: DOWNLOAD_BASE_URL,
    };
    render_typed_page(&engine, "cowork-setup", &data, &user_ctx, &mkt_ctx)
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
