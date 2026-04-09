use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::identifiers::{SessionId, UserId};

use crate::admin::event_hub::EventHub;
use crate::admin::numeric;
use crate::admin::repositories::{conversation_analytics, hooks_track, usage_aggregations};
use crate::admin::types::webhook::{HookEvent, HookEventPayload};

use super::{ai_summary, entity, helpers};

pub(crate) struct ProcessInsertedEventParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub session_id: &'a SessionId,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub payload: &'a HookEventPayload,
    pub event_hub: &'a EventHub,
    pub ai_service: &'a Option<Arc<AiService>>,
    pub jwt_token: &'a str,
    pub tier_cache: &'a crate::admin::tier_enforcement::TierEnforcementCache,
}

pub(crate) async fn process_inserted_event(params: &ProcessInsertedEventParams<'_>) {
    let pool = params.pool;
    let user_id = params.user_id;
    let session_id = params.session_id;
    let event_type = params.event_type;
    let payload = params.payload;
    let today = chrono::Utc::now().date_naive();

    usage_aggregations::upsert_daily_aggregation(&usage_aggregations::DailyAggregationParams {
        pool,
        user_id,
        date: &today,
        event_type,
        tool_name: params.tool_name,
        content_input_bytes: params.content_input_bytes,
        content_output_bytes: params.content_output_bytes,
        is_error: matches!(&payload.event, HookEvent::PostToolUseFailure(_)),
    })
    .await;

    if !session_id.as_str().is_empty() {
        let file_path = helpers::extract_file_path(payload);
        let is_from_subagent = payload.common.agent_id.is_some();
        usage_aggregations::increment_session_summary(&usage_aggregations::SessionSummaryParams {
            pool,
            session_id,
            user_id,
            event_type,
            content_input_bytes: params.content_input_bytes,
            content_output_bytes: params.content_output_bytes,
            is_subagent_stop: matches!(&payload.event, HookEvent::SubagentStop(_)),
            file_path: file_path.as_deref(),
            is_from_subagent,
        })
        .await;

        if event_type == "SessionStart" {
            if let HookEvent::SessionStart(ref data) = payload.event {
                usage_aggregations::update_session_metadata(
                    pool,
                    session_id,
                    &data.source,
                    &data.model,
                    &payload.common.permission_mode,
                )
                .await;
            }
        }

        if !payload.common.permission_mode.is_empty() && event_type != "SessionStart" {
            usage_aggregations::update_session_permission_mode(
                pool,
                session_id,
                &payload.common.permission_mode,
            )
            .await;
        }
    }

    if !session_id.as_str().is_empty() {
        if let Some((entity_type, entity_name)) = entity::detect_entity(payload) {
            let entity_id = if entity_type == "skill" {
                Some(
                    entity_name
                        .rsplit_once(':')
                        .map_or(entity_name.as_str(), |(_, slug)| slug)
                        .to_string(),
                )
            } else {
                None
            };
            if let Err(e) = conversation_analytics::upsert_session_entity_link(
                pool,
                user_id,
                session_id.as_str(),
                entity_type,
                &entity_name,
                entity_id.as_deref(),
            )
            .await
            {
                tracing::warn!(error = %e, "Failed to upsert session entity link");
            }
        }
    }

    handle_prompt_title(pool, event_type, session_id, payload).await;

    if event_type == "Stop" && !session_id.as_str().is_empty() {
        handle_session_analysis(params).await;
        handle_apm_and_concurrent(params).await;
    }

    if event_type == "SessionEnd" && !session_id.as_str().is_empty() {
        handle_session_end(params).await;
    }

    params.event_hub.notify(user_id).await;
}

async fn handle_prompt_title(
    pool: &PgPool,
    event_type: &str,
    session_id: &SessionId,
    payload: &HookEventPayload,
) {
    if event_type != "UserPromptSubmit" || session_id.as_str().is_empty() {
        return;
    }
    if let HookEvent::UserPromptSubmit(ref data) = payload.event {
        if !data.prompt.is_empty() {
            let initial_title = helpers::derive_title(&data.prompt);
            usage_aggregations::update_session_title_if_empty(pool, session_id, &initial_title)
                .await;
        }
    }
}

async fn handle_session_analysis(params: &ProcessInsertedEventParams<'_>) {
    let pool = params.pool;
    let user_id = params.user_id;
    let session_id = params.session_id;
    let payload = params.payload;

    let can_analyse = crate::admin::tier_enforcement::check_limit(
        params.tier_cache,
        pool,
        user_id,
        crate::admin::tier_limits::LimitCheck::FeatureAccess(
            crate::admin::tier_limits::Feature::AiSessionAnalysis,
        ),
    )
    .await;

    if can_analyse.allowed {
        run_ai_analysis(params).await;
    } else {
        tracing::info!(
            user_id = user_id.as_str(),
            session_id = session_id.as_str(),
            "Skipping AI session analysis: tier check denied"
        );
        if let HookEvent::Stop(ref stop_data) = payload.event {
            if let Some(ref msg) = stop_data.last_assistant_message {
                if !msg.is_empty() {
                    let title = helpers::derive_title(msg);
                    let summary = helpers::truncate(msg, 2000);
                    usage_aggregations::update_session_ai_summary_with_title(
                        pool,
                        session_id,
                        Some(&title),
                        &summary,
                        "",
                    )
                    .await;
                }
            }
        }
    }
}

async fn handle_session_end(params: &ProcessInsertedEventParams<'_>) {
    let _ = hooks_track::mark_session_ended(params.pool, params.session_id).await;
}

async fn run_ai_analysis(params: &ProcessInsertedEventParams<'_>) {
    if let Some(ref ai) = params.ai_service {
        let direct_msg = if let HookEvent::Stop(ref d) = params.payload.event {
            d.last_assistant_message
                .as_deref()
                .filter(|m| !m.is_empty())
        } else {
            None
        };
        ai_summary::run_analysis_for_session(
            params.pool,
            ai,
            params.user_id,
            params.session_id,
            params.jwt_token,
            direct_msg,
        )
        .await;
    }
}

async fn handle_apm_and_concurrent(params: &ProcessInsertedEventParams<'_>) {
    let pool = params.pool;
    let user_id = params.user_id;
    let session_id = params.session_id;

    let can_apm = crate::admin::tier_enforcement::check_limit(
        params.tier_cache,
        pool,
        user_id,
        crate::admin::tier_limits::LimitCheck::FeatureAccess(
            crate::admin::tier_limits::Feature::ApmMetrics,
        ),
    )
    .await;

    if !can_apm.allowed {
        return;
    }

    let (apm, eapm) = crate::admin::repositories::apm_metrics::calculate_session_apm(
        pool,
        session_id.as_str(),
    )
    .await;

    let concurrent_raw =
        hooks_track::count_concurrent_sessions(pool, user_id, session_id).await;

    let concurrent = numeric::saturating_i32(concurrent_raw) + 1;

    crate::admin::repositories::apm_metrics::update_session_apm(
        pool,
        session_id.as_str(),
        apm,
        eapm,
        concurrent,
    )
    .await;
}
