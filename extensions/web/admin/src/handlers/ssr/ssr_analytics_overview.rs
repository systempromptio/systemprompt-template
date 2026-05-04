use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::{get_overview, OverviewRow};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

async fn fetch_overview(pool: &PgPool) -> Result<OverviewRow, sqlx::Error> {
    get_overview(pool).await
}

pub async fn analytics_overview_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let row = fetch_overview(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch analytics overview");
        OverviewRow {
            total_events: 0,
            total_sessions: 0,
            tool_uses: 0,
            errors: 0,
        }
    });

    let data = json!({
        "page": "analytics-overview",
        "title": "Analytics Overview",
        "cli_command": "systemprompt analytics overview",
        "window_label": "last 24 hours",
        "stats": {
            "total_events": row.total_events,
            "total_sessions": row.total_sessions,
            "tool_uses": row.tool_uses,
            "errors": row.errors,
        },
    });

    super::render_page(&engine, "analytics-overview", &data, &user_ctx, &mkt_ctx)
}
