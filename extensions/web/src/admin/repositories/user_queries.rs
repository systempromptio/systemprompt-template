use std::sync::Arc;

use sqlx::PgPool;

pub async fn fetch_department_stats(
    pool: &Arc<PgPool>,
) -> Result<Vec<super::super::types::DepartmentStats>, sqlx::Error> {
    sqlx::query_as::<_, super::super::types::DepartmentStats>(
        r"
        SELECT
            COALESCE(NULLIF(u.department, ''), 'Unassigned') AS department,
            COUNT(DISTINCT u.id)::BIGINT AS user_count,
            COUNT(DISTINCT u.id) FILTER (WHERE u.status = 'active')::BIGINT AS active_count,
            COALESCE(SUM(ev.event_count), 0)::BIGINT AS total_events,
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '24 hours')::BIGINT AS active_24h,
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '7 days')::BIGINT AS active_7d,
            COALESCE(SUM(tok.total_tokens), 0)::BIGINT AS total_tokens,
            COALESCE(SUM(ev.prompt_count), 0)::BIGINT AS total_prompts,
            COALESCE(SUM(ev.session_count), 0)::BIGINT AS total_sessions,
            COALESCE(SUM(ev.sessions_this_week), 0)::BIGINT AS sessions_this_week,
            COALESCE(SUM(ev.sessions_prev_week), 0)::BIGINT AS sessions_prev_week
        FROM users u
        LEFT JOIN (
            SELECT
                user_id,
                COUNT(*)::BIGINT AS event_count,
                COUNT(*) FILTER (WHERE event_type LIKE '%UserPromptSubmit%')::BIGINT AS prompt_count,
                COUNT(DISTINCT session_id)::BIGINT AS session_count,
                COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days')::BIGINT AS sessions_this_week,
                COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - INTERVAL '14 days' AND created_at < NOW() - INTERVAL '7 days')::BIGINT AS sessions_prev_week,
                MAX(created_at) AS last_event
            FROM plugin_usage_events
            GROUP BY user_id
        ) ev ON ev.user_id = u.id
        LEFT JOIN (
            SELECT
                user_id,
                (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0))::BIGINT AS total_tokens
            FROM plugin_usage_daily
            GROUP BY user_id
        ) tok ON tok.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY COALESCE(NULLIF(u.department, ''), 'Unassigned')
        ORDER BY user_count DESC
        ",
    )
    .fetch_all(pool.as_ref())
    .await
}
