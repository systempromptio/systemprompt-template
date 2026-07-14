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

use crate::event_hub::EventHub;
use crate::repositories::webhook;
use crate::types::webhook::{HookEvent, HookEventPayload};
use auth::extract_and_validate_jwt;
use axum::Json;
use axum::extract::{Extension, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt::ai::AiService;
use systemprompt::identifiers::{SessionId, UserId};

pub(crate) async fn handle_hook_track(
    Extension(event_hub): Extension<EventHub>,
    Extension(ai_service): Extension<Option<Arc<AiService>>>,
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
    log_payload_warnings(&payload, &warnings);

    let was_inserted = insert_hook_event(&pool, &user_id, &payload).await;
    if !was_inserted {
        tracing::trace!(
            plugin_id = %plugin_id,
            event_type = payload.event_name(),
            "Hook event deduplicated"
        );
        return StatusCode::OK.into_response();
    }

    dispatch_inserted_event(&DispatchContext {
        pool: &pool,
        user_id: &user_id,
        payload: &payload,
        event_hub: &event_hub,
        ai_service: ai_service.as_ref(),
        jwt_token: &jwt_token,
    })
    .await;
    StatusCode::OK.into_response()
}

fn log_payload_warnings(payload: &HookEventPayload, warnings: &[String]) {
    for w in warnings {
        tracing::debug!(
            event_type = payload.event_name(),
            warning = %w,
            "Hook payload validation warning"
        );
    }
}

struct DispatchContext<'a> {
    pool: &'a PgPool,
    user_id: &'a UserId,
    payload: &'a HookEventPayload,
    event_hub: &'a EventHub,
    ai_service: Option<&'a Arc<AiService>>,
    jwt_token: &'a str,
}

async fn dispatch_inserted_event(ctx: &DispatchContext<'_>) {
    let content_bytes = helpers::compute_content_bytes(ctx.payload);
    processing::process_inserted_event(&processing::ProcessInsertedEventParams {
        pool: ctx.pool,
        user_id: ctx.user_id,
        session_id: &SessionId::new(ctx.payload.session_id()),
        event_type: ctx.payload.event_name(),
        tool_name: ctx.payload.tool_name(),
        content_input_bytes: content_bytes.input,
        content_output_bytes: content_bytes.output,
        payload: ctx.payload,
        event_hub: ctx.event_hub,
        ai_service: ctx.ai_service,
        jwt_token: ctx.jwt_token,
    })
    .await;
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
        description: Some(&*description),
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
        },
    }
}
