use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

use crate::admin::repositories;
use crate::admin::types::{HookEventPayload, StatusLinePayload, StatusLineQuery, TrackQuery};

use super::helpers::{
    build_metadata, build_statusline_metadata, extract_bearer_token, get_jwt_config,
    spawn_activity_recording, spawn_aggregation, ActivityRecordingParams, AggregationParams,
};

pub(crate) async fn track_hook_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<TrackQuery>,
    Json(payload): Json<HookEventPayload>,
) -> Response {
    let Some(token) = extract_bearer_token(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})),
        )
            .into_response();
    };

    let (jwt_secret, jwt_issuer) = match get_jwt_config() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response();
        }
    };

    let claims = match systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
        ],
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "Plugin webhook JWT validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})),
            )
                .into_response();
        }
    };

    let user_id = &claims.sub;
    let session_id = payload.session_id.as_deref().unwrap_or("unknown");
    let event_type = payload
        .hook_event_name
        .as_deref()
        .map_or_else(|| "unknown".to_string(), |e| format!("claude_code_{e}"));
    let tool_name = payload.tool_name.as_deref();
    let plugin_id = query.plugin_id.as_deref();
    let metadata = build_metadata(&payload);

    match repositories::insert_plugin_usage_event(
        &pool,
        user_id,
        session_id,
        &event_type,
        tool_name,
        plugin_id,
        &metadata,
    )
    .await
    {
        Ok(inserted) => {
            if inserted {
                spawn_activity_recording(&ActivityRecordingParams {
                    pool: &pool,
                    user_id,
                    event_type: &event_type,
                    session_id,
                    plugin_id,
                    payload: &payload,
                });
                spawn_aggregation(&AggregationParams {
                    pool: &pool,
                    user_id,
                    session_id,
                    event_type: &event_type,
                    tool_name,
                    plugin_id,
                    payload: &payload,
                });
            }
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to insert plugin usage event");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to record event"})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn track_statusline_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<StatusLineQuery>,
    Json(payload): Json<StatusLinePayload>,
) -> Response {
    let Some(token) = extract_bearer_token(&headers) else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Missing Authorization header"})),
        )
            .into_response();
    };

    let (jwt_secret, jwt_issuer) = match get_jwt_config() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal configuration error"})),
            )
                .into_response();
        }
    };

    let claims = match systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
        ],
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "StatusLine webhook JWT validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})),
            )
                .into_response();
        }
    };

    let user_id = &claims.sub;
    let session_id = query.session_id.as_deref().unwrap_or("unknown");
    let plugin_id = query.plugin_id.as_deref();
    let metadata = build_statusline_metadata(&payload);

    let model_id = payload
        .model
        .as_ref()
        .and_then(|m| m.api_model_id.clone());
    let input_tokens = payload
        .context_window
        .as_ref()
        .and_then(|cw| cw.current_usage.as_ref())
        .and_then(|u| u.input_tokens);
    let output_tokens = payload
        .context_window
        .as_ref()
        .and_then(|cw| cw.current_usage.as_ref())
        .and_then(|u| u.output_tokens);

    match repositories::insert_plugin_usage_event(
        &pool,
        user_id,
        session_id,
        "claude_code_StatusLine",
        None,
        plugin_id,
        &metadata,
    )
    .await
    {
        Ok(_inserted) => {
            let p = pool.clone();
            let uid = user_id.to_string();
            let sid = session_id.to_string();
            let mid = model_id.clone();
            let inp = input_tokens;
            let out = output_tokens;
            tokio::spawn(async move {
                repositories::usage_aggregations::update_session_tokens(
                    &p,
                    &sid,
                    &uid,
                    mid.as_deref(),
                    inp,
                    out,
                )
                .await;
            });
            StatusCode::NO_CONTENT.into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to insert StatusLine usage event");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to record event"})),
            )
                .into_response()
        }
    }
}
