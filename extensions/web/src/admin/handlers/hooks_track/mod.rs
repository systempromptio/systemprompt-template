pub(crate) mod ai_context;
pub(crate) mod ai_summary;
pub(crate) mod ai_summary_types;
mod auth;
mod dedup;
mod description;
mod entity;
mod helpers;
mod processing;
pub(crate) mod session_summary;

use crate::admin::event_hub::EventHub;
use crate::admin::repositories::webhook;
use crate::admin::types::webhook::{HookEvent, HookEventPayload};
use auth::extract_and_validate_jwt;
use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::ai::AiService;
use systemprompt::identifiers::{SessionId, UserId};

pub(crate) async fn handle_hook_track(
    Extension(event_hub): Extension<EventHub>,
    Extension(ai_service): Extension<Option<Arc<AiService>>>,
    Extension(tier_cache): Extension<crate::admin::tier_enforcement::TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Json(raw): Json<serde_json::Value>,
) -> Response {
    tracing::info!(payload = %raw, "Hook track received raw payload");
    let (user_id, plugin_id, jwt_token) = match extract_and_validate_jwt(&headers) {
        Ok(ids) => ids,
        Err(r) => return *r,
    };
    let (payload, warnings) = HookEventPayload::from_value(raw);
    if matches!(&payload.event, HookEvent::PreToolUse(_)) {
        return StatusCode::OK.into_response();
    }
    if let Some(resp) = check_event_limit(&tier_cache, &pool, &user_id).await {
        return resp;
    }
    if payload.event_name() == "SessionStart" {
        if let Some(resp) = check_session_limit(&tier_cache, &pool, &user_id).await {
            return resp;
        }
    }
    for w in &warnings {
        tracing::debug!(
            event_type = payload.event_name(),
            warning = %w,
            "Hook payload validation warning"
        );
    }

    let was_inserted = insert_hook_event(&pool, &user_id, &payload).await;

    if !was_inserted {
        tracing::trace!(
            plugin_id = %plugin_id,
            event_type = payload.event_name(),
            "Hook event deduplicated"
        );
    }

    if was_inserted {
        let content_bytes = helpers::compute_content_bytes(&payload);
        processing::process_inserted_event(&processing::ProcessInsertedEventParams {
            pool: &*pool,
            user_id: &user_id,
            session_id: &SessionId::new(payload.session_id()),
            event_type: payload.event_name(),
            tool_name: payload.tool_name(),
            content_input_bytes: content_bytes.input,
            content_output_bytes: content_bytes.output,
            payload: &payload,
            event_hub: &event_hub,
            ai_service: &ai_service,
            jwt_token: &jwt_token,
            tier_cache: &tier_cache,
        })
        .await;
    }

    StatusCode::OK.into_response()
}

async fn insert_hook_event(pool: &PgPool, user_id: &UserId, payload: &HookEventPayload) -> bool {
    let session_id = SessionId::new(payload.session_id());
    let description = description::generate_description(payload);
    let prompt_preview = helpers::generate_prompt_preview(payload);
    let dedup_key = dedup::compute_dedup_key(user_id, &session_id, payload);
    let content_bytes = helpers::compute_content_bytes(payload);
    let sanitized_metadata = helpers::sanitize_metadata(&payload.raw);

    let usage_params = webhook::UsageEventParams {
        user_id,
        session_id: &session_id,
        event_type: payload.event_name(),
        tool_name: payload.tool_name(),
        metadata: &sanitized_metadata,
        description: Some(&description),
        prompt_preview: prompt_preview.as_deref(),
        cwd: payload.cwd(),
        dedup_key: &dedup_key,
        content_input_bytes: content_bytes.input,
        content_output_bytes: content_bytes.output,
    };

    match webhook::insert_plugin_usage_event(pool, &usage_params).await {
        Ok(inserted) => inserted,
        Err(e) => {
            tracing::warn!(error = %e, "Failed to insert hook tracking event");
            false
        }
    }
}

async fn check_event_limit(
    tier_cache: &crate::admin::tier_enforcement::TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> Option<Response> {
    let event_check = crate::admin::tier_enforcement::check_limit(
        tier_cache,
        pool,
        user_id,
        crate::admin::tier_limits::LimitCheck::IngestEvent,
    )
    .await;
    if !event_check.allowed {
        tracing::info!(user_id = %user_id, "Hook event rejected: daily event limit reached");
        return Some(
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "daily_event_limit_reached",
                    "message": event_check.reason,
                    "limit": event_check.limit_value,
                    "current": event_check.current_value,
                })),
            )
                .into_response(),
        );
    }
    None
}

async fn check_session_limit(
    tier_cache: &crate::admin::tier_enforcement::TierEnforcementCache,
    pool: &PgPool,
    user_id: &UserId,
) -> Option<Response> {
    let session_check = crate::admin::tier_enforcement::check_limit(
        tier_cache,
        pool,
        user_id,
        crate::admin::tier_limits::LimitCheck::IngestSession,
    )
    .await;
    if !session_check.allowed {
        tracing::info!(user_id = %user_id, "Hook event rejected: daily session limit reached");
        return Some(
            (
                StatusCode::TOO_MANY_REQUESTS,
                Json(serde_json::json!({
                    "error": "daily_session_limit_reached",
                    "message": session_check.reason,
                    "limit": session_check.limit_value,
                    "current": session_check.current_value,
                })),
            )
                .into_response(),
        );
    }
    None
}
