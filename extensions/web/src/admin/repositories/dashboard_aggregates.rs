use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{ActivityStats, EventTypeBreakdown, TimeSeriesBucket};

pub async fn fetch_event_breakdown(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<EventTypeBreakdown>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, EventTypeBreakdown>(
            r"SELECT p.event_type, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY p.event_type ORDER BY count DESC",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, EventTypeBreakdown>(
            r"SELECT p.event_type, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY p.event_type ORDER BY count DESC",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn get_activity_stats(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<ActivityStats, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, ActivityStats>(
            r"SELECT
                COALESCE(COUNT(*) FILTER (WHERE p.created_at >= CURRENT_DATE), 0)::BIGINT AS events_today,
                COALESCE(COUNT(*) FILTER (WHERE p.created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS events_this_week,
                COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS total_sessions,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type ILIKE '%error%' OR p.event_type ILIKE '%fail%'), 0)::BIGINT AS error_count,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse'), 0)::BIGINT AS tool_uses,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_UserPromptSubmit'), 0)::BIGINT AS prompts,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_SubagentStart'), 0)::BIGINT AS subagents_spawned,
                COALESCE((SELECT SUM(total_input_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_input_tokens,
                COALESCE((SELECT SUM(total_output_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_output_tokens,
                COALESCE((SELECT SUM((p2.metadata->>'total_cost_usd')::NUMERIC) FROM plugin_usage_events p2 WHERE p2.event_type = 'claude_code_StatusLine'), 0.0)::FLOAT8 AS total_cost_usd,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUseFailure'), 0)::BIGINT AS failure_count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1",
        )
        .bind(dept)
        .fetch_one(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, ActivityStats>(
            r"SELECT
                COALESCE(COUNT(*) FILTER (WHERE p.created_at >= CURRENT_DATE), 0)::BIGINT AS events_today,
                COALESCE(COUNT(*) FILTER (WHERE p.created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS events_this_week,
                COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS total_sessions,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type ILIKE '%error%' OR p.event_type ILIKE '%fail%'), 0)::BIGINT AS error_count,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse'), 0)::BIGINT AS tool_uses,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_UserPromptSubmit'), 0)::BIGINT AS prompts,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_SubagentStart'), 0)::BIGINT AS subagents_spawned,
                COALESCE((SELECT SUM(total_input_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_input_tokens,
                COALESCE((SELECT SUM(total_output_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_output_tokens,
                COALESCE((SELECT SUM((p2.metadata->>'total_cost_usd')::NUMERIC) FROM plugin_usage_events p2 WHERE p2.event_type = 'claude_code_StatusLine'), 0.0)::FLOAT8 AS total_cost_usd,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUseFailure'), 0)::BIGINT AS failure_count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'",
        )
        .fetch_one(pool.as_ref())
        .await
    }
}

pub async fn fetch_usage_timeseries(
    pool: &Arc<PgPool>,
    department: Option<&str>,
    interval: &str,
) -> Result<Vec<TimeSeriesBucket>, sqlx::Error> {
    if let Some(dept) = department {
        let sql = format!(
            r"SELECT
                date_trunc('hour', p.created_at) AS bucket,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse'), 0)::BIGINT AS tool_uses,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_UserPromptSubmit'), 0)::BIGINT AS prompts,
                COALESCE(COUNT(DISTINCT p.user_id), 0)::BIGINT AS active_users,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_SessionStart'), 0)::BIGINT AS sessions,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type ILIKE '%fail%' OR p.event_type ILIKE '%error%'), 0)::BIGINT AS errors
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '{interval}'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY bucket
            ORDER BY bucket"
        );
        sqlx::query_as::<_, TimeSeriesBucket>(&sql)
            .bind(dept)
            .fetch_all(pool.as_ref())
            .await
    } else {
        let sql = format!(
            r"SELECT
                date_trunc('hour', p.created_at) AS bucket,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse'), 0)::BIGINT AS tool_uses,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_UserPromptSubmit'), 0)::BIGINT AS prompts,
                COALESCE(COUNT(DISTINCT p.user_id), 0)::BIGINT AS active_users,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_SessionStart'), 0)::BIGINT AS sessions,
                COALESCE(COUNT(*) FILTER (WHERE p.event_type ILIKE '%fail%' OR p.event_type ILIKE '%error%'), 0)::BIGINT AS errors
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '{interval}'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY bucket
            ORDER BY bucket"
        );
        sqlx::query_as::<_, TimeSeriesBucket>(&sql)
            .fetch_all(pool.as_ref())
            .await
    }
}

pub async fn fetch_active_users_24h(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<i64, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_scalar::<_, i64>(
            r"SELECT COALESCE(COUNT(DISTINCT p.user_id), 0)::BIGINT
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '24 hours'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1",
        )
        .bind(dept)
        .fetch_one(pool.as_ref())
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            r"SELECT COALESCE(COUNT(DISTINCT p.user_id), 0)::BIGINT
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '24 hours'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'",
        )
        .fetch_one(pool.as_ref())
        .await
    }
}

pub async fn fetch_avg_session_duration(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<i64, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_scalar::<_, i64>(
            r"SELECT COALESCE(
                EXTRACT(EPOCH FROM AVG(e.created_at - s.created_at))::BIGINT,
                0
            )
            FROM plugin_usage_events s
            JOIN plugin_usage_events e ON e.session_id = s.session_id AND e.event_type = 'claude_code_SessionEnd'
            JOIN users u ON u.id = s.user_id
            WHERE s.event_type = 'claude_code_SessionStart'
              AND s.created_at >= NOW() - INTERVAL '7 days'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1",
        )
        .bind(dept)
        .fetch_one(pool.as_ref())
        .await
    } else {
        sqlx::query_scalar::<_, i64>(
            r"SELECT COALESCE(
                EXTRACT(EPOCH FROM AVG(e.created_at - s.created_at))::BIGINT,
                0
            )
            FROM plugin_usage_events s
            JOIN plugin_usage_events e ON e.session_id = s.session_id AND e.event_type = 'claude_code_SessionEnd'
            JOIN users u ON u.id = s.user_id
            WHERE s.event_type = 'claude_code_SessionStart'
              AND s.created_at >= NOW() - INTERVAL '7 days'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'",
        )
        .fetch_one(pool.as_ref())
        .await
    }
}
