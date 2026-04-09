use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, Copy)]
pub struct DailyAggregationParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub date: &'a chrono::NaiveDate,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub is_error: bool,
}

pub async fn upsert_daily_aggregation(params: &DailyAggregationParams<'_>) {
    let id = format!(
        "{date}_{user_id}_{}_{}",
        params.event_type,
        params.tool_name.unwrap_or(""),
        date = params.date,
        user_id = params.user_id.as_str(),
    );
    let error_inc = i64::from(params.is_error);

    let result = sqlx::query!(
        r"INSERT INTO plugin_usage_daily
            (id, date, user_id, event_type, tool_name, event_count, content_input_bytes, content_output_bytes, error_count)
           VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8)
           ON CONFLICT (date, user_id, event_type, COALESCE(tool_name, ''))
           DO UPDATE SET
             event_count = plugin_usage_daily.event_count + 1,
             content_input_bytes = plugin_usage_daily.content_input_bytes + $6,
             content_output_bytes = plugin_usage_daily.content_output_bytes + $7,
             error_count = plugin_usage_daily.error_count + $8,
             updated_at = NOW()",
        id,
        params.date,
        params.user_id.as_str(),
        params.event_type,
        params.tool_name,
        params.content_input_bytes,
        params.content_output_bytes,
        error_inc,
    )
    .execute(params.pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to upsert daily aggregation");
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SessionSummaryParams<'a> {
    pub pool: &'a PgPool,
    pub session_id: &'a SessionId,
    pub user_id: &'a UserId,
    pub event_type: &'a str,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub is_subagent_stop: bool,
    pub file_path: Option<&'a str>,
    pub is_from_subagent: bool,
}

pub async fn increment_session_summary(params: &SessionSummaryParams<'_>) {
    let is_tool_use =
        params.event_type == "PostToolUse" || params.event_type == "PostToolUseFailure";
    let is_prompt = params.event_type.contains("UserPromptSubmit");
    let is_error = params.event_type.contains("Failure");
    let tool_inc = i64::from(is_tool_use);
    let prompt_inc = i64::from(is_prompt);
    let error_inc = i64::from(is_error);
    let subagent_inc = i64::from(params.is_subagent_stop);
    let user_prompt_inc = i32::from(is_prompt && !params.is_from_subagent);
    let automated_inc = i32::from(is_tool_use && params.is_from_subagent);

    let id = format!("sess_{}", params.session_id.as_str());
    let result = sqlx::query!(
        r"INSERT INTO plugin_session_summaries
            (id, session_id, user_id, total_events, tool_uses, prompts, errors,
             content_input_bytes, content_output_bytes, subagent_spawns,
             user_prompts, automated_actions, started_at)
           VALUES ($1, $2, $3, 1, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
           ON CONFLICT (session_id) DO UPDATE SET
             total_events = plugin_session_summaries.total_events + 1,
             tool_uses = plugin_session_summaries.tool_uses + $4,
             prompts = plugin_session_summaries.prompts + $5,
             errors = plugin_session_summaries.errors + $6,
             content_input_bytes = plugin_session_summaries.content_input_bytes + $7,
             content_output_bytes = plugin_session_summaries.content_output_bytes + $8,
             subagent_spawns = plugin_session_summaries.subagent_spawns + $9,
             user_prompts = plugin_session_summaries.user_prompts + $10,
             automated_actions = plugin_session_summaries.automated_actions + $11,
             updated_at = NOW()",
        id,
        params.session_id.as_str(),
        params.user_id.as_str(),
        tool_inc,
        prompt_inc,
        error_inc,
        params.content_input_bytes,
        params.content_output_bytes,
        subagent_inc,
        user_prompt_inc,
        automated_inc,
    )
    .execute(params.pool)
    .await;

    if let Err(e) = result {
        tracing::debug!(error = %e, "Failed to increment session summary (likely duplicate key)");
    }

    if let Some(fp) = params.file_path {
        if !fp.is_empty() {
            super::session_updates::update_unique_files_touched(params.pool, params.session_id, fp)
                .await;
        }
    }
}
