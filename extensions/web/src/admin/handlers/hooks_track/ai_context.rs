use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::admin::repositories::conversation_analytics;

use super::session_summary;

struct SessionMetricsRow {
    prompts: i64,
    unique_files_touched: Option<i32>,
    errors: i64,
    subagent_spawns: i64,
    client_source: String,
    permission_mode: String,
    model: String,
    user_prompts: Option<i32>,
    automated_actions: Option<i32>,
}

struct SessionTimingRow {
    started: Option<chrono::DateTime<chrono::Utc>>,
    ended: Option<chrono::DateTime<chrono::Utc>>,
}

fn format_user_messages(messages: &[String]) -> Option<String> {
    if messages.is_empty() {
        return None;
    }
    let numbered: Vec<String> = messages
        .iter()
        .enumerate()
        .map(|(i, msg)| format!("{}. \"{}\"", i + 1, msg))
        .collect();
    Some(format!(
        "USER MESSAGES (chronological):\n{}",
        numbered.join("\n")
    ))
}

fn format_skills(skills: &[&str]) -> Option<String> {
    if skills.is_empty() {
        return None;
    }
    Some(format!("SKILLS USED: {}", skills.join(", ")))
}

fn format_session_metrics(m: &SessionMetricsRow) -> Vec<String> {
    let mut parts = Vec::new();
    let files_count = m.unique_files_touched.unwrap_or(0);
    let user_p = m.user_prompts.unwrap_or(0);
    let auto_a = m.automated_actions.unwrap_or(0);

    let mut metadata_parts = Vec::new();
    if !m.client_source.is_empty() {
        metadata_parts.push(format!("Client={}", m.client_source));
    }
    if !m.permission_mode.is_empty() {
        metadata_parts.push(format!("Mode={}", m.permission_mode));
    }
    if !m.model.is_empty() {
        metadata_parts.push(format!("Model={}", m.model));
    }
    if !metadata_parts.is_empty() {
        parts.push(format!("SESSION METADATA: {}", metadata_parts.join(", ")));
    }

    let mut session_parts = Vec::new();
    if user_p > 0 || auto_a > 0 {
        session_parts.push(format!(
            "{user_p} user prompts, {auto_a} automated tool calls"
        ));
    } else {
        session_parts.push(format!("{} prompts", m.prompts));
    }
    session_parts.push(format!("{files_count} files touched"));
    if m.errors > 0 {
        session_parts.push(format!("{} errors", m.errors));
    }
    if m.subagent_spawns > 0 {
        session_parts.push(format!("{} subagents", m.subagent_spawns));
    }
    parts.push(format!("SESSION: {}", session_parts.join(", ")));

    parts
}

fn format_session_timing(t: &SessionTimingRow) -> Option<String> {
    if let (Some(started), Some(ended)) = (t.started, t.ended) {
        let duration_secs = (ended - started).num_seconds().max(0);
        let duration_mins = duration_secs / 60;
        let start_time = started.format("%H:%M");
        let end_time = ended.format("%H:%M");
        Some(format!(
            "SESSION TIMING: Started {start_time}, Ended {end_time} ({duration_mins} minutes)"
        ))
    } else {
        None
    }
}

pub async fn gather_analysis_context(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
) -> String {
    let mut parts = Vec::new();

    let user_messages = sqlx::query_scalar!(
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
    .unwrap_or_else(|_| Vec::new());

    if let Some(msg_part) = format_user_messages(&user_messages) {
        parts.push(msg_part);
    }

    let entity_links =
        conversation_analytics::fetch_session_entity_links(pool, session_id.as_str())
            .await
            .unwrap_or_else(|_| Vec::new());

    let skills: Vec<&str> = entity_links
        .iter()
        .filter(|e| e.entity_type == "skill")
        .map(|e| e.entity_name.as_str())
        .collect();

    if let Some(skills_part) = format_skills(&skills) {
        parts.push(skills_part);
    }

    let metrics = sqlx::query_as!(
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
        tracing::warn!(error = %e, session_id = %session_id.as_str(), "Failed to fetch session metrics for AI context");
    })
    .ok()
    .flatten();

    if let Some(m) = metrics {
        parts.extend(format_session_metrics(&m));
    }

    let timing = sqlx::query_as!(
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
        tracing::warn!(error = %e, session_id = %session_id.as_str(), "Failed to fetch session timing for AI context");
    })
    .ok()
    .flatten();

    if let Some(t) = timing {
        if let Some(timing_part) = format_session_timing(&t) {
            parts.push(timing_part);
        }
    }

    parts.join("\n\n")
}

pub fn build_full_context(
    analysis_context: &str,
    events_ctx: Option<&session_summary::SessionSummary>,
) -> String {
    if let Some(s) = events_ctx {
        let tags_part = if s.tags.is_empty() {
            String::new()
        } else {
            format!("\nTags: {}", s.tags)
        };
        if analysis_context.is_empty() {
            format!("{}{tags_part}", s.summary)
        } else {
            format!("{analysis_context}\nActivity: {}{tags_part}", s.summary)
        }
    } else {
        analysis_context.to_string()
    }
}

pub async fn resolve_last_message(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
    direct_message: Option<&str>,
) -> String {
    if let Some(msg) = direct_message.filter(|m| !m.is_empty()) {
        return msg.to_string();
    }

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
