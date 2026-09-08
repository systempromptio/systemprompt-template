//! `POST /api/public/admin/approvals/{call_id}/{approve|deny}` — deciding a
//! held call.
//!
//! The `require_approval` policy parks a tool call on an `approval_requests`
//! row and blocks its caller on it. Nothing in core exposes the decision over
//! HTTP, so the console adds it here: two routes rather than one with a body,
//! because the verb is the whole payload and a mistyped `{"status":"aprove"}`
//! should not be able to reach a 200.
//!
//! The write is conditional on the row still being pending, so two approvers
//! racing the same queue produce one decision and one 409. Overwriting the
//! first would be the single worst failure this table exists to prevent.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::{AdminError, AdminResult};
use crate::repositories::governance::approvals::{find_approval, update_approval_decision};
use crate::types::UserContext;

const NOTE_LIMIT: usize = 500;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct DecideQuery {
    // Why: Free-text reason, recorded on the row beside the verdict.
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DecisionResponse {
    pub call_id: String,
    pub status: &'static str,
    pub decided_by: String,
}

pub(crate) async fn approve_handler(
    user_ctx: Extension<UserContext>,
    pool: State<Arc<PgPool>>,
    call_id: Path<String>,
    query: Query<DecideQuery>,
) -> AdminResult<Response> {
    decide(user_ctx, pool, call_id, query, "approved").await
}

pub(crate) async fn deny_handler(
    user_ctx: Extension<UserContext>,
    pool: State<Arc<PgPool>>,
    call_id: Path<String>,
    query: Query<DecideQuery>,
) -> AdminResult<Response> {
    decide(user_ctx, pool, call_id, query, "denied").await
}

async fn decide(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(call_id): Path<String>,
    Query(query): Query<DecideQuery>,
    status: &'static str,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }

    let Some(row) = find_approval(&pool, &call_id).await? else {
        return Err(AdminError::NotFound(format!(
            "No approval request {call_id}"
        )));
    };
    if !row.is_actionable() {
        return Err(AdminError::Conflict(format!(
            "Request {call_id} is already {}",
            row.effective_status()
        )));
    }

    let note = query
        .note
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(|n| n.chars().take(NOTE_LIMIT).collect::<String>());

    let changed = update_approval_decision(
        &pool,
        &call_id,
        status,
        &user_ctx.user_id,
        &user_ctx.username,
        note.as_deref(),
    )
    .await?;

    // Why: zero rows means another approver decided it between the read above
    // and this write. The loser is told so rather than being shown a success
    // for a decision that is not theirs.
    if changed == 0 {
        return Err(AdminError::Conflict(format!(
            "Request {call_id} was decided by someone else"
        )));
    }

    Ok((
        StatusCode::OK,
        Json(DecisionResponse {
            call_id,
            status,
            decided_by: user_ctx.username.clone(),
        }),
    )
        .into_response())
}
