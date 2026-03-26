use sqlx::PgPool;

pub struct DailyAggregationParams<'a> {
    pub pool: &'a PgPool,
    pub user_id: &'a str,
    pub date: &'a chrono::NaiveDate,
    pub event_type: &'a str,
    pub tool_name: Option<&'a str>,
    pub plugin_id: Option<&'a str>,
    pub duration_ms: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub is_error: bool,
}

pub async fn upsert_daily_aggregation(params: &DailyAggregationParams<'_>) {
    let id = format!(
        "{date}_{user_id}_{}_{}_{}",
        params.plugin_id.unwrap_or(""),
        params.event_type,
        params.tool_name.unwrap_or(""),
        date = params.date,
        user_id = params.user_id,
    );
    let error_inc = i64::from(params.is_error);
    let dur = params.duration_ms.unwrap_or(0);
    let inp = params.input_tokens.unwrap_or(0);
    let out = params.output_tokens.unwrap_or(0);

    let result = sqlx::query(
        r"INSERT INTO plugin_usage_daily
            (id, date, user_id, plugin_id, event_type, tool_name, event_count, total_duration_ms, total_input_tokens, total_output_tokens, error_count)
           VALUES ($1, $2, $3, $4, $5, $6, 1, $7, $8, $9, $10)
           ON CONFLICT (date, user_id, event_type, COALESCE(plugin_id, ''), COALESCE(tool_name, ''))
           DO UPDATE SET
             event_count = plugin_usage_daily.event_count + 1,
             total_duration_ms = plugin_usage_daily.total_duration_ms + $7,
             total_input_tokens = plugin_usage_daily.total_input_tokens + $8,
             total_output_tokens = plugin_usage_daily.total_output_tokens + $9,
             error_count = plugin_usage_daily.error_count + $10,
             updated_at = NOW()",
    )
    .bind(&id)
    .bind(params.date)
    .bind(params.user_id)
    .bind(params.plugin_id)
    .bind(params.event_type)
    .bind(params.tool_name)
    .bind(dur)
    .bind(inp)
    .bind(out)
    .bind(error_inc)
    .execute(params.pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to upsert daily aggregation");
    }
}

pub async fn upsert_session_summary_start(
    pool: &PgPool,
    session_id: &str,
    user_id: &str,
    plugin_id: Option<&str>,
    model: Option<&str>,
) {
    let id = format!("sess_{session_id}");
    let result = sqlx::query(
        r"INSERT INTO plugin_session_summaries
            (id, session_id, user_id, plugin_id, model, started_at, total_events)
           VALUES ($1, $2, $3, $4, $5, NOW(), 1)
           ON CONFLICT (session_id) DO UPDATE SET
             started_at = COALESCE(plugin_session_summaries.started_at, NOW()),
             model = COALESCE($5, plugin_session_summaries.model),
             total_events = plugin_session_summaries.total_events + 1,
             updated_at = NOW()",
    )
    .bind(&id)
    .bind(session_id)
    .bind(user_id)
    .bind(plugin_id)
    .bind(model)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to upsert session summary start");
    }
}

pub async fn upsert_session_summary_end(pool: &PgPool, session_id: &str) {
    let result = sqlx::query(
        r"UPDATE plugin_session_summaries
           SET ended_at = NOW(), total_events = total_events + 1, updated_at = NOW()
           WHERE session_id = $1",
    )
    .bind(session_id)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to upsert session summary end");
    }
}

pub async fn update_session_tokens(
    pool: &PgPool,
    session_id: &str,
    user_id: &str,
    model: Option<&str>,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
) {
    let inp = input_tokens.unwrap_or(0);
    let out = output_tokens.unwrap_or(0);
    let id = format!("sess_{session_id}");

    let result = sqlx::query(
        r"INSERT INTO plugin_session_summaries
            (id, session_id, user_id, model, total_input_tokens, total_output_tokens, total_events)
           VALUES ($1, $2, $3, $4, $5, $6, 0)
           ON CONFLICT (session_id) DO UPDATE SET
             total_input_tokens = EXCLUDED.total_input_tokens,
             total_output_tokens = EXCLUDED.total_output_tokens,
             model = COALESCE(EXCLUDED.model, plugin_session_summaries.model),
             updated_at = NOW()",
    )
    .bind(&id)
    .bind(session_id)
    .bind(user_id)
    .bind(model)
    .bind(inp)
    .bind(out)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session tokens from StatusLine");
    }
}

pub async fn update_session_tokens_from_transcript(
    pool: &PgPool,
    session_id: &str,
    user_id: &str,
    model: Option<&str>,
    input_tokens: i64,
    output_tokens: i64,
) {
    let id = format!("sess_{session_id}");
    let result = sqlx::query(
        r"INSERT INTO plugin_session_summaries
            (id, session_id, user_id, model, total_input_tokens, total_output_tokens, total_events)
           VALUES ($1, $2, $3, $4, $5, $6, 0)
           ON CONFLICT (session_id) DO UPDATE SET
             total_input_tokens = EXCLUDED.total_input_tokens,
             total_output_tokens = EXCLUDED.total_output_tokens,
             model = COALESCE(EXCLUDED.model, plugin_session_summaries.model),
             updated_at = NOW()",
    )
    .bind(&id)
    .bind(session_id)
    .bind(user_id)
    .bind(model)
    .bind(input_tokens)
    .bind(output_tokens)
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session tokens from transcript");
    }
}

pub struct SessionSummaryParams<'a> {
    pub pool: &'a PgPool,
    pub session_id: &'a str,
    pub user_id: &'a str,
    pub event_type: &'a str,
    pub plugin_id: Option<&'a str>,
    pub model: Option<&'a str>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
}

pub async fn increment_session_summary(params: &SessionSummaryParams<'_>) {
    let is_tool_use = params.event_type.contains("ToolUse");
    let is_prompt = params.event_type.contains("UserPromptSubmit");
    let is_error = params.event_type.contains("Failure");
    let tool_inc = i64::from(is_tool_use);
    let prompt_inc = i64::from(is_prompt);
    let error_inc = i64::from(is_error);
    let inp = params.input_tokens.unwrap_or(0);
    let out = params.output_tokens.unwrap_or(0);

    let id = format!("sess_{}", params.session_id);
    let result = sqlx::query(
        r"INSERT INTO plugin_session_summaries
            (id, session_id, user_id, plugin_id, model, total_events, tool_uses, prompts, errors, total_input_tokens, total_output_tokens)
           VALUES ($1, $2, $3, $4, $5, 1, $6, $7, $8, $9, $10)
           ON CONFLICT (session_id) DO UPDATE SET
             total_events = plugin_session_summaries.total_events + 1,
             tool_uses = plugin_session_summaries.tool_uses + $6,
             prompts = plugin_session_summaries.prompts + $7,
             errors = plugin_session_summaries.errors + $8,
             total_input_tokens = plugin_session_summaries.total_input_tokens + $9,
             total_output_tokens = plugin_session_summaries.total_output_tokens + $10,
             model = COALESCE($5, plugin_session_summaries.model),
             updated_at = NOW()",
    )
    .bind(&id)
    .bind(params.session_id)
    .bind(params.user_id)
    .bind(params.plugin_id)
    .bind(params.model)
    .bind(tool_inc)
    .bind(prompt_inc)
    .bind(error_inc)
    .bind(inp)
    .bind(out)
    .execute(params.pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to increment session summary");
    }
}
