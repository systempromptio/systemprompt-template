use std::sync::Arc;

use sqlx::PgPool;

use super::super::activity;
use super::super::types::{UsageEvent, UserSummary};
use super::user_skills;

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

pub async fn get_user_detail(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Option<super::super::types::UserDetail>, sqlx::Error> {
    let row = sqlx::query_as::<_, UserSummary>(
        r"
        SELECT
            u.id AS user_id,
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            u.email,
            NULLIF(u.department, '') AS department,
            u.roles,
            (u.status = 'active') AS is_active,
            COALESCE(MAX(a.created_at), u.created_at) AS last_active,
            COALESCE(COUNT(DISTINCT a.id), 0)::BIGINT AS total_events,
            (SELECT a2.entity_name FROM user_activity a2
             WHERE a2.user_id = u.id AND a2.entity_name IS NOT NULL
             ORDER BY a2.created_at DESC LIMIT 1) AS last_tool,
            COALESCE(s.skill_count, 0)::BIGINT AS custom_skills_count,
            (SELECT plugin_id FROM plugin_usage_events p3
             WHERE p3.user_id = u.id AND p3.plugin_id IS NOT NULL
             GROUP BY plugin_id ORDER BY COUNT(*) DESC LIMIT 1) AS preferred_client,
            COALESCE(pe.prompt_count, 0)::BIGINT AS prompts,
            COALESCE(pe.session_count, 0)::BIGINT AS sessions,
            COALESCE(tok.total_tokens, 0)::BIGINT AS tokens
        FROM users u
        LEFT JOIN user_activity a ON a.user_id = u.id
        LEFT JOIN (
            SELECT user_id, COUNT(*)::BIGINT AS skill_count
            FROM user_skills GROUP BY user_id
        ) s ON s.user_id = u.id
        LEFT JOIN (
            SELECT user_id,
                   COUNT(*) FILTER (WHERE event_type LIKE '%UserPromptSubmit%')::BIGINT AS prompt_count,
                   COUNT(DISTINCT session_id)::BIGINT AS session_count
            FROM plugin_usage_events GROUP BY user_id
        ) pe ON pe.user_id = u.id
        LEFT JOIN (
            SELECT user_id,
                   (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0))::BIGINT AS total_tokens
            FROM plugin_usage_daily GROUP BY user_id
        ) tok ON tok.user_id = u.id
        WHERE u.id = $1
        GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                 u.roles, u.status, u.department, s.skill_count, pe.prompt_count, pe.session_count, tok.total_tokens
        ",
    )
    .bind(user_id)
    .fetch_optional(pool.as_ref())
    .await?;

    let Some(summary) = row else {
        return Ok(None);
    };

    let skills = user_skills::list_user_skills(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = user_id, error = %e, "Failed to load user skills");
            Vec::new()
        });
    let recent_activity = activity::queries::get_user_recent_activity(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = user_id, error = %e, "Failed to load user activity");
            Vec::new()
        });
    let activity_summary = activity::queries::get_user_activity_summary(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = user_id, error = %e, "Failed to load activity summary");
            Vec::new()
        });

    let created_at: chrono::DateTime<chrono::Utc> =
        sqlx::query_scalar("SELECT created_at FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool.as_ref())
            .await?
            .unwrap_or(summary.last_active);

    Ok(Some(super::super::types::UserDetail {
        user_id: summary.user_id,
        display_name: summary.display_name,
        email: summary.email,
        department: summary.department,
        roles: summary.roles,
        is_active: summary.is_active,
        last_active: summary.last_active,
        total_events: summary.total_events,
        custom_skills_count: summary.custom_skills_count,
        preferred_client: summary.preferred_client,
        created_at,
        skills,
        recent_activity,
        activity_summary,
    }))
}

pub async fn get_user_usage(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UsageEvent>, sqlx::Error> {
    sqlx::query_as::<_, UsageEvent>(
        r"
        SELECT id, event_type, tool_name, plugin_id, created_at, metadata
        FROM plugin_usage_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_distinct_roles(pool: &Arc<PgPool>) -> Result<Vec<String>, sqlx::Error> {
    let rows: Vec<(String,)> = sqlx::query_as(
        r"SELECT DISTINCT unnest(roles) AS role FROM users
          WHERE NOT ('anonymous' = ANY(roles))
          ORDER BY role",
    )
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows
        .into_iter()
        .map(|(r,)| r)
        .filter(|r| !["anonymous", "a2a", "mcp", "service"].contains(&r.as_str()))
        .collect())
}
