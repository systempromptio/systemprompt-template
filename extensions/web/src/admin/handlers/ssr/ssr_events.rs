use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{EventsQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub async fn events_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let (response, event_breakdown) = tokio::join!(
        repositories::list_events(&pool, &query),
        repositories::list_event_breakdown(&pool),
    );
    let response = response.unwrap_or_else(|_| crate::admin::types::EventsResponse {
        events: vec![],
        total: 0,
        limit: query.limit,
        offset: query.offset,
    });
    let event_breakdown = event_breakdown.unwrap_or_else(|_| vec![]);

    let has_prev = response.offset > 0;
    let has_next = response.offset + response.limit < response.total;
    let prev_offset = if response.offset >= response.limit {
        response.offset - response.limit
    } else {
        0
    };
    let next_offset = response.offset + response.limit;

    let data = json!({
        "page": "events",
        "title": "Events",
        "events": response.events,
        "total": response.total,
        "limit": response.limit,
        "offset": response.offset,
        "has_prev": has_prev,
        "has_next": has_next,
        "prev_offset": prev_offset,
        "next_offset": next_offset,
        "search": query.search,
        "event_type": query.event_type,
        "event_breakdown": event_breakdown,
    });
    super::render_page(&engine, "events", &data, &user_ctx, &mkt_ctx)
}
