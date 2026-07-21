//! Conversation transcript capture from webhook payloads.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::types::webhook::{TranscriptPayload, TranscriptQuery};

use super::helpers::authenticate_webhook;

pub(crate) async fn track_transcript_event(
    State(_pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(_query): Query<TranscriptQuery>,
    Json(_payload): Json<TranscriptPayload>,
) -> AdminResult<Response> {
    authenticate_webhook(&headers)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
