use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::repositories;

pub(crate) async fn list_jobs_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::list_jobs(&pool).await {
        Ok(jobs) => Json(jobs).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list jobs");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
