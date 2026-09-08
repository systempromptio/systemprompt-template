//! Reading and overriding a user's primary group and project.
//!
//! The recomputation job picks both from membership and rewrites its own
//! answer whenever membership moves. Writing here marks the row `manual`,
//! which is what stops the job from overwriting it again — so an operator's
//! choice is permanent until they change it, not until the next run.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::error::{AdminError, AdminResult};
use crate::repositories::scope::defaults::{self, ScopeDefaults};
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub(crate) struct SetScopeDefaultsRequest {
    pub primary_group_id: Option<String>,
    pub primary_project_id: Option<String>,
}

pub(crate) async fn get_user_scope_defaults_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id);
    require_user(&pool, &user_id).await?;
    let found = defaults::find_scope_defaults(&pool, &user_id).await?;
    let body = found.unwrap_or_else(|| ScopeDefaults {
        primary_group_id: None,
        primary_project_id: None,
        source: "auto".to_owned(),
    });
    Ok(Json(body).into_response())
}

pub(crate) async fn set_user_scope_defaults_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
    Json(body): Json<SetScopeDefaultsRequest>,
) -> AdminResult<Response> {
    let user_id = UserId::new(user_id);
    require_user(&pool, &user_id).await?;
    let written = defaults::set_scope_defaults(
        &pool,
        &user_id,
        body.primary_group_id.as_deref(),
        body.primary_project_id.as_deref(),
    )
    .await?;
    tracing::info!(
        actor = %user_ctx.user_id.as_str(),
        user_id = %user_id.as_str(),
        "Scope defaults set manually"
    );
    Ok(Json(written).into_response())
}

// Why: Recompute every `auto` attribution key from current membership.
//
// The keys are only rewritten where membership is written through this API.
// The directory replaces a signer-in's whole `adfs` membership set at each
// sign-in without passing through it, and a restore or an import writes the
// tables directly, so an estate drifts: people keep their groups and lose
// their primary group, and their spend silently becomes unattributed. This is
// the operator's way back, and it leaves `manual` rows alone.
pub(crate) async fn recompute_scope_defaults_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let written = defaults::recompute_scope_defaults(&pool).await?;
    tracing::info!(
        actor = %user_ctx.user_id.as_str(),
        written,
        "Scope defaults recomputed"
    );
    Ok(Json(serde_json::json!({ "recomputed": written })).into_response())
}

// Why: without this the insert fails on the foreign key and the caller is told
// the server broke, when what happened is that they named a user who is gone.
async fn require_user(pool: &PgPool, user_id: &UserId) -> AdminResult<()> {
    sqlx::query_scalar!("SELECT id FROM users WHERE id = $1", user_id.as_str())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AdminError::NotFound(format!("User {user_id} not found")))?;
    Ok(())
}
