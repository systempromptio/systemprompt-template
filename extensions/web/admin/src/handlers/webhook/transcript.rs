//! Conversation transcript capture from webhook payloads.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use systemprompt::models::auth::JwtAudience;

use crate::handlers::shared;
use crate::types::webhook::{TranscriptPayload, TranscriptQuery};

use super::helpers::{extract_bearer_token, get_jwt_issuer};

pub(crate) async fn track_transcript_event(
    State(_pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(_query): Query<TranscriptQuery>,
    Json(_payload): Json<TranscriptPayload>,
) -> Response {
    let Some(token) = extract_bearer_token(&headers) else {
        return shared::error_response(StatusCode::UNAUTHORIZED, "Missing Authorization header");
    };

    let jwt_issuer = match get_jwt_issuer() {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load JWT config");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            );
        },
    };

    if let Err(e) = systemprompt::oauth::validate_jwt_token(
        token,
        &jwt_issuer,
        &[
            JwtAudience::Resource("hook".to_owned()),
            JwtAudience::Resource("plugin".to_owned()),
            JwtAudience::Api,
        ],
    ) {
        tracing::warn!(error = %e, "Transcript webhook JWT validation failed");
        return shared::error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token");
    }

    StatusCode::NO_CONTENT.into_response()
}
