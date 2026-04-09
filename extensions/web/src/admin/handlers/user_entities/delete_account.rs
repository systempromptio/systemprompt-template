use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::admin::handlers::shared;
use crate::admin::repositories;
use crate::admin::types::UserContext;

pub async fn delete_account_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> Response {
    let user_id = &user_ctx.user_id;

    match repositories::delete_user_complete(&pool, user_id).await {
        Ok(true) => {
            tracing::info!(user_id = %user_id, "User self-deleted account");
            let mut response = StatusCode::NO_CONTENT.into_response();
            if let Ok(cookie) =
                HeaderValue::from_str("access_token=; Path=/; SameSite=Lax; Max-Age=0")
            {
                response.headers_mut().insert("set-cookie", cookie);
            }
            response
        }
        Ok(false) => shared::error_response(StatusCode::NOT_FOUND, "Account not found"),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to delete account");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete account",
            )
        }
    }
}
