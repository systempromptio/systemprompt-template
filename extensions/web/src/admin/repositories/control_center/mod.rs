mod sessions;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub use crate::admin::types::control_center::TodayStats;
use crate::admin::types::control_center::{ActivityFeedEvent, RecentSession, RecentTask};

pub use sessions::{fetch_recent_sessions_filtered, update_session_status};

pub async fn fetch_activity_feed(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<ActivityFeedEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityFeedEvent,
        r"SELECT
            id, session_id, event_type, tool_name,
            description, prompt_preview, cwd,
            created_at
        FROM plugin_usage_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2",
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_recent_sessions(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<RecentSession>, sqlx::Error> {
    fetch_recent_sessions_filtered(pool, user_id, limit, "").await
}

pub async fn fetch_session_events(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    session_ids: &[String],
) -> Result<Vec<ActivityFeedEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityFeedEvent,
        r"SELECT
            id, session_id, event_type, tool_name,
            description, prompt_preview, cwd,
            created_at
        FROM plugin_usage_events
        WHERE user_id = $1 AND session_id = ANY($2)
        ORDER BY created_at DESC",
        user_id.as_str(),
        session_ids,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_today_stats(pool: &Arc<PgPool>, user_id: &UserId) -> TodayStats {
    let sessions_started = sqlx::query_scalar!(
        r#"SELECT COALESCE(COUNT(*), 0)::BIGINT as "count!"
         FROM plugin_session_summaries
         WHERE user_id = $1 AND started_at >= CURRENT_DATE"#,
        user_id.as_str(),
    )
    .fetch_one(pool.as_ref())
    .await
    .unwrap_or(0);

    let daily = fetch_daily_stats(pool, user_id).await;

    TodayStats {
        sessions_started,
        total_prompts: daily.total_prompts,
        total_tool_calls: daily.total_tool_calls,
        total_errors: daily.total_errors,
        content_input_bytes: daily.content_input_bytes,
        content_output_bytes: daily.content_output_bytes,
    }
}

struct DailyRow {
    total_prompts: i64,
    total_tool_calls: i64,
    total_errors: i64,
    content_input_bytes: i64,
    content_output_bytes: i64,
}

async fn fetch_daily_stats(pool: &Arc<PgPool>, user_id: &UserId) -> DailyRow {
    #[derive(sqlx::FromRow)]
    struct DailyQueryRow {
        total_prompts: i64,
        total_tool_calls: i64,
        total_errors: i64,
        content_input_bytes: i64,
        content_output_bytes: i64,
    }

    sqlx::query_as!(
        DailyQueryRow,
        r#"SELECT
            COALESCE(SUM(event_count) FILTER (WHERE event_type = 'UserPromptSubmit'), 0)::BIGINT AS "total_prompts!",
            COALESCE(SUM(event_count) FILTER (WHERE event_type = 'PostToolUse'), 0)::BIGINT AS "total_tool_calls!",
            COALESCE(SUM(error_count), 0)::BIGINT AS "total_errors!",
            COALESCE(SUM(content_input_bytes), 0)::BIGINT AS "content_input_bytes!",
            COALESCE(SUM(content_output_bytes), 0)::BIGINT AS "content_output_bytes!"
        FROM plugin_usage_daily
        WHERE user_id = $1 AND date = CURRENT_DATE"#,
        user_id.as_str(),
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id.as_str(), "Failed to fetch daily usage row for control center");
    })
    .ok()
    .flatten()
    .map_or(
        DailyRow {
            total_prompts: 0,
            total_tool_calls: 0,
            total_errors: 0,
            content_input_bytes: 0,
            content_output_bytes: 0,
        },
        |r| DailyRow {
            total_prompts: r.total_prompts,
            total_tool_calls: r.total_tool_calls,
            total_errors: r.total_errors,
            content_input_bytes: r.content_input_bytes,
            content_output_bytes: r.content_output_bytes,
        },
    )
}

pub struct TodayOutcomeStats {
    pub completed_today: i64,
    pub positive_count: i64,
    pub rated_count: i64,
}

pub async fn fetch_today_outcome_stats(pool: &Arc<PgPool>, user_id: &UserId) -> TodayOutcomeStats {
    #[derive(sqlx::FromRow)]
    struct Row {
        completed_today: i64,
        positive_count: i64,
        rated_count: i64,
    }

    sqlx::query_as!(
        Row,
        r#"SELECT
            COALESCE(COUNT(*) FILTER (WHERE s.ended_at IS NOT NULL), 0)::BIGINT AS "completed_today!",
            COALESCE(COUNT(*) FILTER (WHERE sr.outcome IN ('success', 'partial')), 0)::BIGINT AS "positive_count!",
            COALESCE(COUNT(*) FILTER (WHERE sr.outcome IS NOT NULL AND sr.outcome != ''), 0)::BIGINT AS "rated_count!"
        FROM plugin_session_summaries s
        LEFT JOIN session_ratings sr ON sr.session_id = s.session_id AND sr.user_id = s.user_id
        WHERE s.user_id = $1 AND s.started_at >= CURRENT_DATE"#,
        user_id.as_str(),
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id.as_str(), "Failed to fetch today's outcome stats");
    })
    .ok()
    .flatten()
    .map_or(
        TodayOutcomeStats { completed_today: 0, positive_count: 0, rated_count: 0 },
        |r| TodayOutcomeStats {
            completed_today: r.completed_today,
            positive_count: r.positive_count,
            rated_count: r.rated_count,
        },
    )
}

pub async fn fetch_recent_tasks(
    pool: &Arc<PgPool>,
    user_id: &UserId,
) -> Result<Vec<RecentTask>, sqlx::Error> {
    sqlx::query_as!(
        RecentTask,
        r"SELECT
            metadata->>'task_subject' AS task_subject,
            metadata->>'task_description' AS task_description,
            created_at
        FROM plugin_usage_events
        WHERE user_id = $1
          AND event_type LIKE '%TaskCompleted%'
          AND metadata->>'task_subject' IS NOT NULL
        ORDER BY created_at DESC
        LIMIT 5",
        user_id.as_str(),
    )
    .fetch_all(pool.as_ref())
    .await
}

#[must_use]
pub fn format_bytes(n: i64) -> String {
    use crate::admin::numeric;

    if n == 0 {
        "0 B".to_string()
    } else if n < 1_024 {
        format!("{n} B")
    } else if n < 1_048_576 {
        format!("{:.1} KB", numeric::to_f64(n) / 1_024.0)
    } else if n < 1_073_741_824 {
        format!("{:.1} MB", numeric::to_f64(n) / 1_048_576.0)
    } else {
        format!("{:.1} GB", numeric::to_f64(n) / 1_073_741_824.0)
    }
}
