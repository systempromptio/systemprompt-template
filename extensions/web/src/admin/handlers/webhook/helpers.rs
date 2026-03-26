use std::sync::Arc;

use axum::http::HeaderMap;
use sqlx::PgPool;
use systemprompt::models::{Config, SecretsBootstrap};

use crate::admin::repositories;
use crate::admin::types::HookEventPayload;

pub(super) use super::activity_recording::{spawn_activity_recording, ActivityRecordingParams};
pub(super) use super::metadata::{build_metadata, build_statusline_metadata};

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| {
            v.to_str()
                .map_err(|e| {
                    tracing::warn!(error = %e, "Non-ASCII authorization header");
                })
                .ok()
        })
        .and_then(|v| v.strip_prefix("Bearer "))
}

pub(super) fn get_jwt_config() -> Result<(String, String), anyhow::Error> {
    let secret = SecretsBootstrap::jwt_secret()?.to_string();
    let issuer = Config::get()?.jwt_issuer.clone();
    Ok((secret, issuer))
}

pub(super) struct AggregationParams<'a> {
    pub pool: &'a Arc<PgPool>,
    pub user_id: &'a str,
    pub session_id: &'a str,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
    pub payload: &'a HookEventPayload,
}

pub(super) fn spawn_aggregation(params: &AggregationParams<'_>) {
    let pool = params.pool;
    let user_id = params.user_id;
    let session_id = params.session_id;
    let event_type = params.event_type;
    let tool_name = params.tool_name;
    let plugin_id = params.plugin_id;
    let payload = params.payload;
    let p = pool.clone();
    let uid = user_id.to_string();
    let et = event_type.to_string();
    let tn = tool_name.map(str::to_string);
    let pid = plugin_id.map(str::to_string);
    let dur = payload.duration_ms;
    let inp = payload.input_tokens;
    let out = payload.output_tokens;
    let success = payload.success;
    let model = payload.model.clone();
    let sid = session_id.to_string();
    tokio::spawn(async move {
        let today = chrono::Utc::now().date_naive();
        let is_error = success.is_some_and(|s| !s);
        repositories::usage_aggregations::upsert_daily_aggregation(
            &repositories::usage_aggregations::DailyAggregationParams {
                pool: &p,
                user_id: &uid,
                date: &today,
                event_type: &et,
                tool_name: tn.as_deref(),
                plugin_id: pid.as_deref(),
                duration_ms: dur,
                input_tokens: inp,
                output_tokens: out,
                is_error,
            },
        )
        .await;

        if et == "claude_code_SessionStart" {
            repositories::usage_aggregations::upsert_session_summary_start(
                &p,
                &sid,
                &uid,
                pid.as_deref(),
                model.as_deref(),
            )
            .await;
        } else if et == "claude_code_SessionEnd" {
            repositories::usage_aggregations::upsert_session_summary_end(&p, &sid).await;
        } else {
            repositories::usage_aggregations::increment_session_summary(
                &repositories::usage_aggregations::SessionSummaryParams {
                    pool: &p,
                    session_id: &sid,
                    user_id: &uid,
                    event_type: &et,
                    plugin_id: pid.as_deref(),
                    model: model.as_deref(),
                    input_tokens: inp,
                    output_tokens: out,
                },
            )
            .await;
        }
    });
}
