//! `GET /admin/api/chain/:id` — JSON envelope used by the chain-drawer.
//!
//! `:id` may be a `decision_id`, `request_id`, `trace_id`, or `session_id`; the
//! repository resolves it to a `session_id` and returns the full chain of
//! custody (decisions, `ai_requests`, plugin events, transcript, summary).

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::repositories::governance_grp::chain::fetch_decision_chain;
use crate::types::UserContext;

pub async fn chain_envelope(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> Response {
    if !user_ctx.is_admin {
        return StatusCode::FORBIDDEN.into_response();
    }

    match fetch_decision_chain(&pool, &id).await {
        Ok(Some(envelope)) => Json(envelope).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            tracing::error!(error = %e, id = %id, "fetch_decision_chain failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
