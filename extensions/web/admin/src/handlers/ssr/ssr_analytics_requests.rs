use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::{get_request_stats, RequestStatsRow};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

async fn fetch_stats(pool: &PgPool) -> Result<RequestStatsRow, sqlx::Error> {
    get_request_stats(pool).await
}

pub async fn analytics_requests_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let stats = fetch_stats(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch request analytics");
        RequestStatsRow {
            total: 0,
            tool_uses: 0,
            errors: 0,
            sessions: 0,
        }
    });

    let data = json!({
        "page": "analytics-requests",
        "title": "Request Analytics",
        "cli_command": "systemprompt analytics requests stats",
        "window_label": "last 24 hours",
        "stats": {
            "total": stats.total,
            "tool_uses": stats.tool_uses,
            "errors": stats.errors,
            "sessions": stats.sessions,
        },
    });

    super::render_page(&engine, "analytics-requests", &data, &user_ctx, &mkt_ctx)
}
