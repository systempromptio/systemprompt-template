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

pub async fn governance_rate_limits_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    // Rate limit config is surfaced via the CLI (`admin config rate-limits show`).
    // We render a read-only page pointing at the authoritative CLI source.
    let tiers = vec![
        json!({ "name": "free", "requests_per_minute": 10, "burst": 20 }),
        json!({ "name": "pro", "requests_per_minute": 60, "burst": 120 }),
        json!({ "name": "enterprise", "requests_per_minute": 600, "burst": 1200 }),
    ];

    let data = json!({
        "page": "governance-rate-limits",
        "title": "Rate Limits",
        "hero_title": "Rate Limit Tiers",
        "hero_subtitle": "Rate limit configuration — authoritative source is the CLI",
        "cli_command": "systemprompt admin config rate-limits show",
        "cli_compare": "systemprompt admin config rate-limits compare",
        "tiers": tiers,
        "has_tiers": true,
    });

    super::render_page(&engine, "governance-rate-limits", &data, &user_ctx, &mkt_ctx)
}
