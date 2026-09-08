//! `POST /admin/api/profile/salesforce/unlink` — drop the caller's Salesforce
//! username mapping.

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use std::sync::Arc;

use crate::authz;
use crate::error::AdminResult;
use crate::repositories::users::salesforce_identity;
use crate::types::UserContext;

pub(crate) async fn salesforce_unlink(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<sqlx::PgPool>>,
) -> AdminResult<Response> {
    // Why: an absent mapping is not an error — the caller asked for it gone and
    // it is gone.
    salesforce_identity::delete_identity(&pool, &user_ctx.user_id).await?;
    authz::salesforce::invalidate(&user_ctx.user_id).await;
    tracing::info!(user_id = %user_ctx.user_id, "Salesforce identity unlinked");
    Ok(Json(serde_json::json!({ "unlinked": true })).into_response())
}
