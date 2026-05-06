//! `GET /admin/api/conversations/:session_id/raw` — capability-gated raw bodies.
//!
//! Returns the un-redacted text of every turn for a session, keyed by ordinal.
//! Refused with 403 unless the caller passes `can_view_raw_transcript`.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::auth::can_view_raw_transcript;
use crate::repositories::analytics_grp::{fetch_raw_turns, RawTurnBody};
use crate::types::UserContext;

#[derive(Debug, Serialize)]
pub struct RawTranscriptEnvelope {
    pub session_id: String,
    pub turns: Vec<RawTurnBody>,
}

pub async fn conversations_raw(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }
    if !can_view_raw_transcript(&user_ctx) {
        return StatusCode::FORBIDDEN.into_response();
    }

    match fetch_raw_turns(&pool, &session_id).await {
        Ok(Some(turns)) => Json(RawTranscriptEnvelope {
            session_id,
            turns,
        })
        .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, session_id = %session_id, "fetch_raw_turns failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
