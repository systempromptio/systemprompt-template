use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

pub async fn mark_session_ended(pool: &PgPool, session_id: &SessionId) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE plugin_session_summaries SET ended_at = NOW(), status = 'completed' WHERE session_id = $1 AND ended_at IS NULL",
        session_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn count_concurrent_sessions(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
) -> i64 {
    sqlx::query_scalar!(
        "SELECT COUNT(*)::BIGINT FROM plugin_session_summaries WHERE user_id = $1 AND started_at <= NOW() AND (ended_at IS NULL OR ended_at >= NOW()) AND session_id != $2",
        user_id.as_str(),
        session_id.as_str(),
    )
    .fetch_one(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or(0)
}

#[derive(sqlx::FromRow, Debug)]
pub struct EventRow {
    pub event_type: String,
    pub tool_name: Option<String>,
    pub cwd: Option<String>,
}

pub async fn fetch_session_events(
    pool: &PgPool,
    session_id: &SessionId,
    user_id: &UserId,
) -> Result<Vec<EventRow>, sqlx::Error> {
    sqlx::query_as::<_, EventRow>(
        r"SELECT event_type, tool_name, cwd
          FROM plugin_usage_events
          WHERE session_id = $1 AND user_id = $2
          ORDER BY created_at ASC",
    )
    .bind(session_id.as_str())
    .bind(user_id.as_str())
    .fetch_all(pool)
    .await
}

pub async fn fetch_user_messages(
    pool: &PgPool,
    session_id: &SessionId,
    user_id: &UserId,
) -> Vec<String> {
    sqlx::query_scalar!(
        r#"SELECT prompt_preview as "prompt_preview!"
          FROM plugin_usage_events
          WHERE session_id = $1 AND user_id = $2 AND event_type = 'UserPromptSubmit'
            AND prompt_preview IS NOT NULL AND prompt_preview != ''
          ORDER BY created_at ASC
          LIMIT 20"#,
        session_id.as_str(),
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

#[derive(Debug)]
pub struct SessionMetricsRow {
    pub prompts: i64,
    pub unique_files_touched: Option<i32>,
    pub errors: i64,
    pub subagent_spawns: i64,
    pub client_source: String,
    pub permission_mode: String,
    pub model: String,
    pub user_prompts: Option<i32>,
    pub automated_actions: Option<i32>,
}

pub async fn fetch_session_metrics(
    pool: &PgPool,
    session_id: &SessionId,
) -> Option<SessionMetricsRow> {
    sqlx::query_as!(
        SessionMetricsRow,
        r#"SELECT prompts, unique_files_touched,
                  COALESCE(errors, 0)::BIGINT AS "errors!",
                  COALESCE(subagent_spawns, 0)::BIGINT AS "subagent_spawns!",
                  COALESCE(client_source, '') AS "client_source!",
                  COALESCE(permission_mode, '') AS "permission_mode!",
                  COALESCE(model, '') AS "model!",
                  user_prompts,
                  automated_actions
          FROM plugin_session_summaries
          WHERE session_id = $1"#,
        session_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, session_id = %session_id.as_str(), "Failed to fetch session metrics");
    })
    .ok()
    .flatten()
}

#[derive(Debug, Clone, Copy)]
pub struct SessionTimingRow {
    pub started: Option<chrono::DateTime<chrono::Utc>>,
    pub ended: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn fetch_session_timing(
    pool: &PgPool,
    session_id: &SessionId,
    user_id: &UserId,
) -> Option<SessionTimingRow> {
    sqlx::query_as!(
        SessionTimingRow,
        r#"SELECT MIN(created_at) AS started, MAX(created_at) AS ended
          FROM plugin_usage_events
          WHERE session_id = $1 AND user_id = $2"#,
        session_id.as_str(),
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, session_id = %session_id.as_str(), "Failed to fetch session timing");
    })
    .ok()
    .flatten()
}

pub async fn fetch_last_message(pool: &PgPool, session_id: &SessionId, user_id: &UserId) -> String {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(prompt_preview, description, '') as "msg!"
          FROM plugin_usage_events
          WHERE session_id = $1 AND user_id = $2
            AND event_type IN ('Stop', 'SubagentStop', 'SessionEnd')
          ORDER BY created_at DESC
          LIMIT 1"#,
        session_id.as_str(),
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, session_id = %session_id.as_str(), "Failed to resolve last message");
    })
    .ok()
    .flatten()
    .unwrap_or_else(String::new)
}

pub async fn fetch_today_achievements(pool: &PgPool, user_id: &str) -> Vec<String> {
    sqlx::query_scalar::<_, String>(
        "SELECT achievement_id FROM user_achievements WHERE user_id = $1 AND unlocked_at::date = CURRENT_DATE",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}
