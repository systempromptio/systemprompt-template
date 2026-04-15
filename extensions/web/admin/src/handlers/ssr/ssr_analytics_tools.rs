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

struct ToolRow {
    tool_name: String,
    calls: i64,
    errors: i64,
    sessions: i64,
}

async fn fetch_tools(pool: &PgPool) -> Result<Vec<ToolRow>, sqlx::Error> {
    sqlx::query_as!(
        ToolRow,
        r#"SELECT
            tool_name AS "tool_name!",
            COUNT(*)::bigint AS "calls!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            COUNT(DISTINCT session_id)::bigint AS "sessions!"
          FROM plugin_usage_events
          WHERE tool_name IS NOT NULL
            AND created_at >= NOW() - INTERVAL '7 days'
          GROUP BY tool_name
          ORDER BY COUNT(*) DESC
          LIMIT 50"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn analytics_tools_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let rows = fetch_tools(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch tool analytics");
        vec![]
    });

    let rows_json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let success_rate = if r.calls > 0 {
                100.0 - (r.errors as f64 * 100.0 / r.calls as f64)
            } else {
                0.0
            };
            json!({
                "tool_name": r.tool_name,
                "calls": r.calls,
                "errors": r.errors,
                "sessions": r.sessions,
                "success_rate": format!("{:.1}", success_rate),
            })
        })
        .collect();

    let data = json!({
        "page": "analytics-tools",
        "title": "Tool Analytics",
        "cli_command": "systemprompt analytics tools list",
        "window_label": "last 7 days",
        "tools": rows_json,
        "has_tools": !rows_json.is_empty(),
    });

    super::render_page(&engine, "analytics-tools", &data, &user_ctx, &mkt_ctx)
}
