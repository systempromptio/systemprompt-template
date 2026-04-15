use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{EventsQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct InfraLogsQuery {
    pub tab: Option<String>,
    pub search: Option<String>,
    pub event_type: Option<String>,
}

pub async fn infra_logs_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<InfraLogsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let tab = query
        .tab
        .clone()
        .unwrap_or_else(|| "view".to_string());

    let events_query = EventsQuery {
        search: query.search.clone(),
        event_type: query.event_type.clone(),
        limit: 50,
        offset: 0,
    };

    let (events_result, breakdown_result) = tokio::join!(
        repositories::list_events(&pool, &events_query),
        repositories::list_event_breakdown(&pool),
    );
    let events = events_result.map_or_else(
        |e| {
            tracing::warn!(error = %e, "Failed to fetch infra logs events");
            vec![]
        },
        |r| r.events,
    );
    let breakdown = breakdown_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch event breakdown");
        vec![]
    });

    let data = json!({
        "page": "infra-logs",
        "title": "Infrastructure — Logs",
        "cli_command": "systemprompt infra logs view --since 1h --limit 10",
        "tab": tab,
        "is_tab_view": tab == "view",
        "is_tab_trace": tab == "trace",
        "is_tab_request": tab == "request",
        "is_tab_tools": tab == "tools",
        "search": query.search,
        "event_type": query.event_type,
        "events": events,
        "has_events": !events.is_empty(),
        "breakdown": breakdown,
    });
    super::render_page(&engine, "infra-logs", &data, &user_ctx, &mkt_ctx)
}
