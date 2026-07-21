//! `GET /admin/api/conversations/:session_id/raw` — capability-gated raw
//! bodies.
//!
//! Returns the un-redacted text of every turn for a session, keyed by ordinal.
//! Refused with 403 unless the caller is admin or holds the `auditor` role.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::analytics::{RawTurnBody, find_raw_turns};
use crate::types::UserContext;

#[derive(Debug, Serialize)]
pub(super) struct RawTranscriptEnvelope {
    pub session_id: SessionId,
    pub turns: Vec<RawTurnBody>,
}

pub(crate) async fn conversations_raw(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(session_id): Path<String>,
) -> AdminResult<Response> {
    let session_id = SessionId::new(session_id);
    let allowed = user_ctx.is_admin
        || user_ctx
            .roles
            .iter()
            .any(|r| r.eq_ignore_ascii_case("auditor"));
    if !allowed {
        return Err(AdminError::Forbidden(
            "Admin or auditor role required".to_owned(),
        ));
    }

    let turns = find_raw_turns(&pool, &session_id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("No transcript for session {session_id}")))?;
    Ok(Json(RawTranscriptEnvelope { session_id, turns }).into_response())
}
