use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::handlers::shared;
use crate::admin::repositories;

use super::responses::JobsListResponse;

pub async fn list_jobs_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::list_jobs(&pool).await {
        Ok(jobs) => Json(JobsListResponse { jobs }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list jobs");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
