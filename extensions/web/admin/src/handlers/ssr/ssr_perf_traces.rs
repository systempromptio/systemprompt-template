use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

struct TraceSessionRow {
    session_id: String,
    event_count: i64,
    tool_uses: i64,
    errors: i64,
    first_at: DateTime<Utc>,
    last_at: DateTime<Utc>,
}

/// SSR page for `/admin/performance/traces` — lists recent request traces
/// (sessions) backing `demo/performance/01-request-tracing.sh`. Clicking
/// through navigates to the existing `/admin/traces` session-detail view.
pub async fn perf_traces_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let traces = fetch_recent_traces(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list recent traces");
        vec![]
    });

    // JSON: template context for Handlebars rendering
    let traces_json: Vec<serde_json::Value> = traces
        .iter()
        .map(|t| {
            let duration_ms = (t.last_at - t.first_at).num_milliseconds();
            let short = if t.session_id.len() > 16 {
                t.session_id[..16].to_string()
            } else {
                t.session_id.clone()
            };
            json!({
                "session_id": t.session_id,
                "session_id_short": short,
                "event_count": t.event_count,
                "tool_uses": t.tool_uses,
                "errors": t.errors,
                "duration_ms": duration_ms,
                "started_at": t.first_at,
            })
        })
        .collect();

    let total_events: i64 = traces.iter().map(|t| t.event_count).sum();
    let total_errors: i64 = traces.iter().map(|t| t.errors).sum();

    let data = json!({
        "page": "perf-traces",
        "title": "Request Traces",
        "cli_command": "systemprompt infra logs trace list --limit 20",
        "demo_script": "demo/performance/01-request-tracing.sh",
        "page_stats": [
            {"key": "traces", "value": traces_json.len(), "label": "Traces"},
            {"key": "events", "value": total_events, "label": "Events"},
            {"key": "errors", "value": total_errors, "label": "Errors"},
        ],
        "traces": traces_json,
        "has_traces": !traces_json.is_empty(),
    });
    super::render_page(&engine, "perf-traces", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_recent_traces(pool: &PgPool) -> Result<Vec<TraceSessionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            session_id AS "session_id!",
            COUNT(*)::bigint AS "event_count!",
            COUNT(*) FILTER (WHERE event_type LIKE '%ToolUse%')::bigint AS "tool_uses!",
            COUNT(*) FILTER (WHERE event_type LIKE '%Failure%' OR event_type LIKE '%Error%')::bigint AS "errors!",
            MIN(created_at) AS "first_at!",
            MAX(created_at) AS "last_at!"
        FROM plugin_usage_events
        WHERE session_id IS NOT NULL
          AND created_at >= NOW() - INTERVAL '7 days'
        GROUP BY session_id
        ORDER BY MAX(created_at) DESC
        LIMIT 50"#,
    )
    .fetch_all(pool)
    .await?;
    let out: Vec<TraceSessionRow> = rows
        .into_iter()
        .map(|r| TraceSessionRow {
            session_id: r.session_id,
            event_count: r.event_count,
            tool_uses: r.tool_uses,
            errors: r.errors,
            first_at: r.first_at,
            last_at: r.last_at,
        })
        .collect();
    Ok(out)
}
