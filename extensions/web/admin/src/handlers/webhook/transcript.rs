//! Conversation transcript capture from webhook payloads.
//!
//! The Claude Code Stop hook posts the full session transcript here; the
//! latest capture per session is what the identity-scoped history surface
//! (`/admin/history`) searches. Persistence is fire-and-forget like the other
//! tracking webhooks: the hook must never surface a storage hiccup to the
//! client, so errors log and the response is 204 regardless.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::analytics::conversations::upsert_session_transcript;
use crate::types::webhook::{TranscriptPayload, TranscriptQuery};

use super::helpers::authenticate_webhook;

pub(crate) async fn track_transcript_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<TranscriptQuery>,
    Json(payload): Json<TranscriptPayload>,
) -> AdminResult<Response> {
    let user_id = authenticate_webhook(&headers)?;

    if let Some(session_id) = payload.session_id.as_ref() {
        let plugin_id = query.plugin_id.as_ref().map(|p| p.as_str());
        if let Err(e) =
            upsert_session_transcript(&pool, &user_id, session_id, plugin_id, &payload.transcript)
                .await
        {
            tracing::warn!(error = %e, "Failed to persist session transcript");
        }
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}
