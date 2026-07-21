//! Session and activity tracking driven by webhook events.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::types::webhook::{StatusLinePayload, StatusLineQuery};

use super::helpers::authenticate_webhook;

pub(crate) async fn track_statusline_event(
    State(_pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(_query): Query<StatusLineQuery>,
    Json(_payload): Json<StatusLinePayload>,
) -> AdminResult<Response> {
    tokio::task::yield_now().await;
    authenticate_webhook(&headers)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
