//! The user index listing and its filter options.

use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use crate::types::UserSummary;
use crate::util::org_scope::OrgScope;

// Why: `admin` is the role a *customer's own* administrator holds, so a
// listing that spans every organization hands them every user on a pooled
// instance. That is why the scope is a parameter rather than a filter the
// caller applies afterwards.
pub async fn list_users(pool: &PgPool, scope: &OrgScope) -> Result<Vec<UserSummary>, sqlx::Error> {
    sqlx::query_as!(
        UserSummary,
        r#"SELECT
                u.id AS "user_id!: UserId",
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email AS "email?: Email",
                u.roles AS "roles!: Vec<String>",
                (u.status = 'active') AS "is_active!",
                -- Why no COALESCE to u.created_at: it made a user who has
                -- never done anything report their join date as their last
                -- activity, so "provisioned but never used" was indistinguishable
                -- from "used on the day they joined" -- which is exactly the
                -- population REQ-005's wasted-seat reporting is about. Postgres
                -- GREATEST ignores NULLs and yields NULL only when every input
                -- is NULL, which is precisely "never active".
                GREATEST(
                    MAX(p.created_at),
                    ua.last_ua,
                    mcp.last_mcp,
                    air.last_request
                ) AS "last_active?",
                (COALESCE(COUNT(DISTINCT p.id), 0) + COALESCE(air.request_count, 0))::BIGINT AS "total_events!",
                (SELECT tool_name FROM plugin_usage_events p2
                 WHERE p2.user_id = u.id
                 ORDER BY created_at DESC LIMIT 1) AS last_tool,
                0::BIGINT AS "custom_skills_count!",
                NULL::TEXT AS preferred_client,
                COALESCE(COUNT(DISTINCT p.id) FILTER (WHERE p.event_type LIKE '%UserPromptSubmit%'), 0)::BIGINT AS "prompts!",
                COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS "sessions!",
                (COALESCE(bytes.total_bytes, 0))::BIGINT AS "bytes!",
                COALESCE(ua.logins, 0)::BIGINT AS "logins!"
            FROM users u
            LEFT JOIN plugin_usage_events p ON p.user_id = u.id
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
            LEFT JOIN (
                SELECT user_id, MAX(created_at) AS last_request, COUNT(*)::BIGINT AS request_count
                FROM ai_requests GROUP BY user_id
            ) air ON air.user_id = u.id
            LEFT JOIN organization_members om ON om.user_id = u.id
            LEFT JOIN organizations o ON o.id = om.org_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND ($1::TEXT IS NULL OR o.slug = $1)
            GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                     u.roles, u.status, bytes.total_bytes,
                     ua.logins, ua.last_ua, mcp.last_mcp, air.last_request,
                     air.request_count
            ORDER BY 6 DESC NULLS LAST"#,
        scope.as_slug(),
    )
    .fetch_all(pool)
    .await
}

pub async fn list_distinct_roles(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT DISTINCT unnest(roles) AS "role!" FROM users
          WHERE NOT ('anonymous' = ANY(roles))
          ORDER BY 1"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| r.role)
        .filter(|r| !["anonymous", "a2a", "mcp", "service"].contains(&r.as_str()))
        .collect())
}
