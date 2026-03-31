use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

use crate::admin::types::webhook::{TranscriptPayload, TranscriptQuery};

use super::helpers::{extract_bearer_token, get_jwt_config};

pub(crate) async fn track_transcript_event(
    State(_pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(_query): Query<TranscriptQuery>,
    Json(_payload): Json<TranscriptPayload>,
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

    if let Err(e) = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_secret,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_string()),
            JwtAudience::Resource("plugin".to_string()),
            JwtAudience::Api,
        ],
    ) {
        tracing::warn!(error = %e, "Transcript webhook JWT validation failed");
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired token"})),
        )
            .into_response();
    }

    StatusCode::NO_CONTENT.into_response()
}
