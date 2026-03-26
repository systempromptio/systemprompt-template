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
use crate::admin::types::TranscriptPayload;
use crate::admin::types::TranscriptQuery;

use super::helpers::{extract_bearer_token, get_jwt_config};

pub(crate) async fn track_transcript_event(
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Query(query): Query<TranscriptQuery>,
    Json(payload): Json<TranscriptPayload>,
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
            tracing::warn!(error = %e, "Transcript webhook JWT validation failed");
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Invalid or expired token"})),
            )
                .into_response();
        }
    };

    let user_id = &claims.sub;
    let session_id = payload.session_id.as_deref().unwrap_or("unknown");
    let plugin_id = query.plugin_id.as_deref();

    let p = pool.clone();
    let uid = user_id.to_string();
    let sid = session_id.to_string();
    let pid = plugin_id.map(str::to_string);
    let transcript = payload.transcript;
    tokio::spawn(async move {
        let skip_count = repositories::get_session_entries_counted(p.as_ref(), &sid)
            .await
            .unwrap_or(0);

        let tokens = repositories::extract_transcript_tokens(&transcript, skip_count);

        let transcript_id = match repositories::insert_session_transcript(
            &p,
            &uid,
            &sid,
            pid.as_deref(),
            &transcript,
        )
        .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::error!(error = %e, "Failed to insert session transcript");
                return;
            }
        };

        let (prev_input, prev_output) =
            get_previous_transcript_totals(p.as_ref(), &sid, &transcript_id).await;
        let total_input = prev_input + tokens.input_tokens;
        let total_output = prev_output + tokens.output_tokens;

        if let Err(e) = repositories::update_transcript_tokens(
            p.as_ref(),
            &transcript_id,
            total_input,
            total_output,
            tokens.model.as_deref(),
            tokens.entries_processed,
        )
        .await
        {
            tracing::error!(error = %e, "Failed to update transcript tokens");
        }

        if total_input > 0 || total_output > 0 {
            repositories::usage_aggregations::update_session_tokens_from_transcript(
                p.as_ref(),
                &sid,
                &uid,
                tokens.model.as_deref(),
                total_input,
                total_output,
            )
            .await;
        }
    });

    StatusCode::NO_CONTENT.into_response()
}

async fn get_previous_transcript_totals(
    pool: &PgPool,
    session_id: &str,
    exclude_id: &str,
) -> (i64, i64) {
    let row: Option<(i64, i64)> = sqlx::query_as(
        "SELECT COALESCE(total_input_tokens, 0), COALESCE(total_output_tokens, 0)
         FROM session_transcripts
         WHERE session_id = $1 AND id != $2
         ORDER BY captured_at DESC
         LIMIT 1",
    )
    .bind(session_id)
    .bind(exclude_id)
    .fetch_optional(pool)
    .await
    .unwrap_or(None);

    row.unwrap_or((0, 0))
}
