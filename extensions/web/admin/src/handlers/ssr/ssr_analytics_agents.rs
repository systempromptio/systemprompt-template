use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::{list_agents, AgentRow};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

async fn fetch_agents(pool: &PgPool) -> Result<Vec<AgentRow>, sqlx::Error> {
    list_agents(pool).await
}

pub async fn analytics_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let rows = fetch_agents(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch agent analytics");
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
                "agent_id": r.agent_id,
                "calls": r.calls,
                "errors": r.errors,
                "sessions": r.sessions,
                "success_rate": format!("{:.1}", success_rate),
            })
        })
        .collect();

    let data = json!({
        "page": "analytics-agents",
        "title": "Agent Analytics",
        "cli_command": "systemprompt analytics agents list",
        "window_label": "last 7 days",
        "agents": rows_json,
        "has_agents": !rows_json.is_empty(),
    });

    super::render_page(&engine, "analytics-agents", &data, &user_ctx, &mkt_ctx)
}
