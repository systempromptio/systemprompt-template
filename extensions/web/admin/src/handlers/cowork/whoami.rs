use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::handlers::shared;

use super::load_user_section;
use super::types::{COWORK_CAPABILITIES, WhoamiResponse};

pub async fn handle(State(pool): State<Arc<PgPool>>, headers: HeaderMap) -> Response {
    let user_id = match super::validate_cowork_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return *r,
    };

    let user = match load_user_section(&pool, &user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        },
        Err(e) => {
            tracing::error!(error = %e, "user lookup failed");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "User lookup failed",
            );
        },
    };

    Json(WhoamiResponse {
        user,
        capabilities: COWORK_CAPABILITIES.to_vec(),
    })
    .into_response()
}
