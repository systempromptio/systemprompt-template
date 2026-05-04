use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::services::jobs_service;

use super::responses::JobsListResponse;

pub async fn list_jobs_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match jobs_service::list_jobs(&pool).await {
        Ok(jobs) => Json(JobsListResponse { jobs }).into_response(),
        Err(e) => e.into_response(),
    }
}
