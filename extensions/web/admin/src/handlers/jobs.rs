//! HTTP handlers for inspecting and triggering scheduled jobs.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::services::jobs_service;

use super::responses::JobsListResponse;

pub(crate) async fn list_jobs_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match jobs_service::list_jobs(&pool).await {
        Ok(jobs) => Json(JobsListResponse { jobs }).into_response(),
        Err(e) => e.into_response(),
    }
}
