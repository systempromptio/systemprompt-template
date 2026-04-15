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

struct RequestStatsRow {
    total: i64,
    tool_uses: i64,
    errors: i64,
    sessions: i64,
}

async fn fetch_stats(pool: &PgPool) -> Result<RequestStatsRow, sqlx::Error> {
    sqlx::query_as!(
        RequestStatsRow,
        r#"SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            COUNT(DISTINCT session_id)::bigint AS "sessions!"
          FROM plugin_usage_events
          WHERE created_at >= NOW() - INTERVAL '24 hours'"#,
    )
    .fetch_one(pool)
    .await
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
