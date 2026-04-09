use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use super::super::super::super::activity;
use super::super::super::super::types::{EventTypeCount, ToolUsageCount, UserSession, UserSummary};
use super::super::super::user_skills;

pub async fn find_user_detail(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<super::super::super::super::types::UserDetail>, sqlx::Error> {
    let row = sqlx::query_as!(
        UserSummary,
        r#"SELECT
            u.id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            u.email as "email?: Email",
            u.roles as "roles!: Vec<String>",
            (u.status = 'active') AS "is_active!",
            GREATEST(
                COALESCE(MAX(a.created_at), u.created_at),
                COALESCE(pe.last_pe, u.created_at),
                COALESCE(mcp.last_mcp, u.created_at)
            ) AS "last_active!",
            COALESCE(COUNT(DISTINCT a.id), 0)::BIGINT AS "total_events!",
            (SELECT a2.entity_name FROM user_activity a2
             WHERE a2.user_id = u.id AND a2.entity_name IS NOT NULL
             ORDER BY a2.created_at DESC LIMIT 1) AS last_tool,
            COALESCE(s.skill_count, 0)::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            COALESCE(pe.prompt_count, 0)::BIGINT AS "prompts!",
            COALESCE(pe.session_count, 0)::BIGINT AS "sessions!",
            COALESCE(bytes.total_bytes, 0)::BIGINT AS "bytes!",
            COALESCE(COUNT(DISTINCT a.id) FILTER (WHERE a.category = 'login'), 0)::BIGINT AS "logins!"
        FROM users u
        LEFT JOIN user_activity a ON a.user_id = u.id
        LEFT JOIN (
            SELECT user_id, COUNT(*)::BIGINT AS skill_count
            FROM user_skills GROUP BY user_id
        ) s ON s.user_id = u.id
        LEFT JOIN (
            SELECT user_id,
                   COUNT(*) FILTER (WHERE event_type LIKE '%UserPromptSubmit%')::BIGINT AS prompt_count,
                   COUNT(DISTINCT session_id)::BIGINT AS session_count,
                   MAX(created_at) AS last_pe
            FROM plugin_usage_events GROUP BY user_id
        ) pe ON pe.user_id = u.id
        LEFT JOIN (
            SELECT user_id,
                   (COALESCE(SUM(content_input_bytes), 0) + COALESCE(SUM(content_output_bytes), 0))::BIGINT AS total_bytes
            FROM plugin_usage_daily GROUP BY user_id
        ) bytes ON bytes.user_id = u.id
        LEFT JOIN (
            SELECT user_id, MAX(created_at) AS last_mcp
            FROM mcp_tool_executions WHERE user_id IS NOT NULL
            GROUP BY user_id
        ) mcp ON mcp.user_id = u.id
        WHERE u.id = $1
        GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                 u.roles, u.status, s.skill_count, pe.prompt_count, pe.session_count,
                 bytes.total_bytes, pe.last_pe, mcp.last_mcp"#,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let Some(summary) = row else {
        return Ok(None);
    };

    build_user_detail(pool, user_id, summary).await
}

async fn build_user_detail(
    pool: &PgPool,
    user_id: &UserId,
    summary: UserSummary,
) -> Result<Option<super::super::super::super::types::UserDetail>, sqlx::Error> {
    let skills = user_skills::list_user_skills(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = %user_id, error = %e, "Failed to load user skills");
            Vec::new()
        });
    let recent_activity = activity::queries::get_user_recent_activity(pool, user_id.as_str())
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = %user_id, error = %e, "Failed to load user activity");
            Vec::new()
        });
    let activity_summary = activity::queries::get_user_activity_summary(pool, user_id.as_str())
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = %user_id, error = %e, "Failed to load activity summary");
            Vec::new()
        });
    let sessions = get_user_sessions(pool, user_id).await.unwrap_or_else(|e| {
        tracing::warn!(user_id = %user_id, error = %e, "Failed to load user sessions");
        Vec::new()
    });
    let event_type_breakdown = get_user_event_type_breakdown(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(user_id = %user_id, error = %e, "Failed to load event type breakdown");
            Vec::new()
        });
    let top_tools = get_user_top_tools(pool, user_id).await.unwrap_or_else(|e| {
        tracing::warn!(user_id = %user_id, error = %e, "Failed to load top tools");
        Vec::new()
    });

    let created_at: chrono::DateTime<chrono::Utc> = sqlx::query_scalar!(
        "SELECT created_at FROM users WHERE id = $1",
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?
    .unwrap_or(summary.last_active);

    Ok(Some(super::super::super::super::types::UserDetail {
        user_id: summary.user_id,
        display_name: summary.display_name,
        email: summary.email,
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
        sessions,
        event_type_breakdown,
        top_tools,
    }))
}

pub async fn get_user_sessions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserSession>, sqlx::Error> {
    sqlx::query_as!(
        UserSession,
        r#"SELECT
            session_id as "session_id!",
            started_at,
            total_events::BIGINT as "total_events!",
            tool_uses::BIGINT as "tool_uses!",
            prompts::BIGINT as "prompts!",
            errors::BIGINT as "errors!",
            COALESCE(content_input_bytes, 0)::BIGINT as "content_input_bytes!",
            COALESCE(content_output_bytes, 0)::BIGINT as "content_output_bytes!",
            COALESCE(subagent_spawns, 0)::BIGINT as "subagent_spawns!"
        FROM plugin_session_summaries
        WHERE user_id = $1
        ORDER BY COALESCE(started_at, created_at) DESC
        LIMIT 20"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn get_user_event_type_breakdown(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<EventTypeCount>, sqlx::Error> {
    sqlx::query_as!(
        EventTypeCount,
        r#"SELECT
            event_type as "event_type!",
            SUM(event_count)::BIGINT as "count!"
        FROM plugin_usage_daily
        WHERE user_id = $1
        GROUP BY event_type
        ORDER BY 2 DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn get_user_top_tools(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ToolUsageCount>, sqlx::Error> {
    sqlx::query_as!(
        ToolUsageCount,
        r#"SELECT
            tool_name as "tool_name!",
            SUM(event_count)::BIGINT as "count!"
        FROM plugin_usage_daily
        WHERE user_id = $1 AND tool_name IS NOT NULL AND tool_name != ''
        GROUP BY tool_name
        ORDER BY 2 DESC
        LIMIT 10"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
