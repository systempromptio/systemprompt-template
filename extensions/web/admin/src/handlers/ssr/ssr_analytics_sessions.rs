use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::{list_sessions, SessionRow};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

async fn fetch_sessions(pool: &PgPool) -> Result<Vec<SessionRow>, sqlx::Error> {
    list_sessions(pool).await
}

pub async fn analytics_sessions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let rows = fetch_sessions(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch session analytics");
        vec![]
    });

    let rows_json: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            json!({
                "session_id": r.session_id,
                "user_id": r.user_id.clone().unwrap_or_else(|| "-".to_string()),
                "events": r.events,
                "first_seen": r.first_seen.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
                "last_seen": r.last_seen.map(|t| t.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string()).unwrap_or_default(),
            })
        })
        .collect();

    let data = json!({
        "page": "analytics-sessions",
        "title": "Session Analytics",
        "cli_command": "systemprompt analytics sessions stats",
        "window_label": "last 24 hours",
        "sessions": rows_json,
        "has_sessions": !rows_json.is_empty(),
        "total_sessions": rows_json.len(),
    });

    super::render_page(&engine, "analytics-sessions", &data, &user_ctx, &mkt_ctx)
}
