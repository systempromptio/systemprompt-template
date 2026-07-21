//! `GET /admin/api/chain/:id` — JSON envelope used by the chain-drawer.
//!
//! `:id` may be a `decision_id`, `request_id`, `trace_id`, or `session_id`; the
//! repository resolves it to a `session_id` and returns the full chain of
//! custody (decisions, `ai_requests`, plugin events, transcript, summary).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::governance::chain::find_decision_chain;
use crate::types::UserContext;

/// The drawer is a JSON consumer, so this returns [`AdminResult`] rather than
/// the HTML face the surrounding page handlers use.
pub(crate) async fn chain_envelope(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let envelope = find_decision_chain(&pool, &id)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("No audit chain for id {id}")))?;

    Ok(Json(envelope).into_response())
}
