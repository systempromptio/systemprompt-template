//! Session and activity tracking driven by webhook events.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};
use crate::repositories::dashboard::usage_aggregations;
use crate::types::webhook::{StatusLineIngest, StatusLinePayload, StatusLineQuery};
use axum::Extension;

use super::helpers::authenticate_webhook;

pub(crate) async fn track_statusline_event(
    Extension(env): Extension<AdminEnv>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<StatusLineQuery>,
    Json(payload): Json<StatusLinePayload>,
) -> AdminResult<Response> {
    let user_id = authenticate_webhook(&env, &headers).await?;
    let ingest = StatusLineIngest::try_from((query, payload))
        .map_err(|rejection| AdminError::BadRequest(rejection.to_string()))?;

    let snapshot = ingest.snapshot(&user_id);
    usage_aggregations::upsert_session_cost_snapshot(&pool, &snapshot).await?;
    usage_aggregations::set_session_summary_tokens(&pool, &snapshot).await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}
