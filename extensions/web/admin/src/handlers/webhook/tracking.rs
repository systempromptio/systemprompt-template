//! Session and activity tracking driven by webhook events.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::dashboard::usage_aggregations;
use crate::types::webhook::{StatusLinePayload, StatusLineQuery};

use super::helpers::authenticate_webhook;

const MICRODOLLARS_PER_DOLLAR: f64 = 1_000_000.0;

pub(crate) async fn track_statusline_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<StatusLineQuery>,
    Json(payload): Json<StatusLinePayload>,
) -> AdminResult<Response> {
    let user_id = authenticate_webhook(&headers)?;

    // Why: persistence stays fire-and-forget like the hooks_track path — the
    // statusline fires constantly and a storage hiccup must not surface to
    // the client, so errors log and the response is 204 regardless.
    if let Some(session_id) = query.session_id.as_ref() {
        let model = payload.model.as_ref().and_then(|m| m.api_model_id.clone());
        let cost = payload
            .cost
            .and_then(|c| c.total_cost_usd)
            .map(usd_to_microdollars);
        let window = payload.context_window;
        let usage = window.and_then(|w| w.current_usage);

        let snapshot = usage_aggregations::SessionCostSnapshot {
            session_id,
            user_id: &user_id,
            model: model.as_deref(),
            total_cost_microdollars: cost,
            context_window_size: window.and_then(|w| w.context_window_size),
            input_tokens: usage.and_then(|u| u.input),
            output_tokens: usage.and_then(|u| u.output),
            cache_creation_input_tokens: usage.and_then(|u| u.cache_creation_input),
            cache_read_input_tokens: usage.and_then(|u| u.cache_read_input),
        };
        if let Err(e) = usage_aggregations::upsert_session_cost_snapshot(&pool, &snapshot).await {
            tracing::warn!(error = %e, "Failed to upsert session cost snapshot");
        }
        if let Err(e) = usage_aggregations::set_session_summary_tokens(
            &pool,
            session_id,
            usage.and_then(|u| u.input),
            usage.and_then(|u| u.output),
            model.as_deref(),
        )
        .await
        {
            tracing::warn!(error = %e, "Failed to set session summary tokens");
        }
    }

    Ok(StatusCode::NO_CONTENT.into_response())
}

fn usd_to_microdollars(usd: f64) -> i64 {
    let micro = usd * MICRODOLLARS_PER_DOLLAR;
    if micro.is_finite() {
        // Why: round-to-nearest keeps sub-cent costs; clamp guards absurd inputs.
        micro.round().clamp(0.0, i64::MAX as f64) as i64
    } else {
        0
    }
}
