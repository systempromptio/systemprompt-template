use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use super::super::super::super::types::UserSummary;

pub async fn list_users(pool: &Arc<PgPool>) -> Result<Vec<UserSummary>, sqlx::Error> {
    sqlx::query_as!(
        UserSummary,
        r#"SELECT
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email",
                u.roles AS "roles!: Vec<String>",
                (u.status = 'active') AS "is_active!",
                GREATEST(
                    COALESCE(MAX(p.created_at), u.created_at),
                    COALESCE(ua.last_ua, u.created_at),
                    COALESCE(mcp.last_mcp, u.created_at)
                ) AS "last_active!",
                COALESCE(COUNT(DISTINCT p.id), 0)::BIGINT AS "total_events!",
                (SELECT tool_name FROM plugin_usage_events p2
                 WHERE p2.user_id = u.id
                 ORDER BY created_at DESC LIMIT 1) AS last_tool,
                COALESCE(s.skill_count, 0)::BIGINT AS "custom_skills_count!",
                NULL::TEXT AS preferred_client,
                COALESCE(COUNT(DISTINCT p.id) FILTER (WHERE p.event_type LIKE '%UserPromptSubmit%'), 0)::BIGINT AS "prompts!",
                COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS "sessions!",
                (COALESCE(bytes.total_bytes, 0))::BIGINT AS "bytes!",
                COALESCE(ua.logins, 0)::BIGINT AS "logins!"
            FROM users u
            LEFT JOIN plugin_usage_events p ON p.user_id = u.id
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS skill_count
                FROM user_skills GROUP BY user_id
            ) s ON s.user_id = u.id
            LEFT JOIN (
                SELECT user_id,
                       (COALESCE(SUM(content_input_bytes), 0) + COALESCE(SUM(content_output_bytes), 0))::BIGINT AS total_bytes
                FROM plugin_usage_daily GROUP BY user_id
            ) bytes ON bytes.user_id = u.id
            LEFT JOIN (
                SELECT user_id,
                       COUNT(*) FILTER (WHERE category = 'login')::BIGINT AS logins,
                       MAX(created_at) AS last_ua
                FROM user_activity GROUP BY user_id
            ) ua ON ua.user_id = u.id
            LEFT JOIN (
                SELECT user_id, MAX(created_at) AS last_mcp
                FROM mcp_tool_executions WHERE user_id IS NOT NULL
                GROUP BY user_id
            ) mcp ON mcp.user_id = u.id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                     u.roles, u.status, s.skill_count, bytes.total_bytes,
                     ua.logins, ua.last_ua, mcp.last_mcp
            ORDER BY 6 DESC"#,
    )
    .fetch_all(pool.as_ref())
    .await
}

#[derive(sqlx::FromRow)]
pub struct UserRank {
    pub user_id: String,
    pub rank_name: String,
    pub total_xp: i64,
}

pub async fn fetch_user_ranks(pool: &Arc<PgPool>) -> Result<Vec<UserRank>, sqlx::Error> {
    sqlx::query_as!(
        UserRank,
        r#"SELECT user_id as "user_id!", rank_name as "rank_name!", total_xp::BIGINT AS "total_xp!" FROM user_ranks"#,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_user_roles(pool: &Arc<PgPool>, user_id: &str) -> Option<Vec<String>> {
    let row = sqlx::query!("SELECT roles FROM users WHERE id = $1", user_id,)
        .fetch_optional(pool.as_ref())
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user roles");
        })
        .ok()
        .flatten()?;

    Some(row.roles)
}

pub async fn fetch_distinct_roles(pool: &Arc<PgPool>) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT DISTINCT unnest(roles) AS "role!" FROM users
          WHERE NOT ('anonymous' = ANY(roles))
          ORDER BY 1"#,
    )
    .fetch_all(pool.as_ref())
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| r.role)
        .filter(|r| !["anonymous", "a2a", "mcp", "service"].contains(&r.as_str()))
        .collect())
}
