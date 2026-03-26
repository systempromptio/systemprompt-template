use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{Email, UserId};

use super::super::activity;
use super::super::types::{ActivityStats, HourlyActivity, SkillCount, ToolSuccessRate, TopUser};

pub async fn fetch_timeline(
    pool: &Arc<PgPool>,
) -> Result<Vec<activity::ActivityTimelineEvent>, sqlx::Error> {
    activity::queries::fetch_timeline(pool, None).await
}

pub async fn fetch_top_users(pool: &Arc<PgPool>) -> Result<Vec<TopUser>, sqlx::Error> {
    sqlx::query_as!(
        TopUser,
        r#"SELECT
            combined.user_id as "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, combined.user_id) AS "display_name!",
            u.email as "email: Email",
            COALESCE(combined.logins, 0)::BIGINT AS "logins!",
            COALESCE(combined.edits, 0)::BIGINT AS "edits!",
            COALESCE(mcp.mcp_calls, 0)::BIGINT AS "mcp_calls!",
            GREATEST(
                COALESCE(combined.last_active, '1970-01-01'::timestamptz),
                COALESCE(mcp.last_mcp, '1970-01-01'::timestamptz),
                COALESCE(pue.last_pue, '1970-01-01'::timestamptz)
            ) AS "last_active!"
        FROM (
            SELECT user_id,
                COUNT(*) FILTER (WHERE category = 'login')::BIGINT AS logins,
                COUNT(*) FILTER (WHERE category = 'marketplace_edit')::BIGINT AS edits,
                MAX(created_at) AS last_active
            FROM user_activity
            GROUP BY user_id
        ) combined
        JOIN users u ON u.id = combined.user_id
        LEFT JOIN (
            SELECT user_id,
                COUNT(*)::BIGINT AS mcp_calls,
                MAX(created_at) AS last_mcp
            FROM mcp_tool_executions
            WHERE user_id IS NOT NULL
            GROUP BY user_id
        ) mcp ON mcp.user_id = combined.user_id
        LEFT JOIN (
            SELECT user_id, MAX(created_at) AS last_pue
            FROM plugin_usage_events
            GROUP BY user_id
        ) pue ON pue.user_id = combined.user_id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        ORDER BY (COALESCE(combined.edits, 0) + COALESCE(mcp.mcp_calls, 0)) DESC, logins DESC
        LIMIT 10"#,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_popular_skills(pool: &Arc<PgPool>) -> Result<Vec<SkillCount>, sqlx::Error> {
    sqlx::query_as!(
        SkillCount,
        r#"SELECT COALESCE(m.tool_name, 'unknown') AS "tool_name!", COUNT(*)::BIGINT AS "count!"
        FROM mcp_tool_executions m
        WHERE m.tool_name IS NOT NULL
        GROUP BY m.tool_name ORDER BY 2 DESC LIMIT 10"#,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_hourly_activity(pool: &Arc<PgPool>) -> Result<Vec<HourlyActivity>, sqlx::Error> {
    sqlx::query_as!(
        HourlyActivity,
        r#"SELECT hour as "hour!", SUM(cnt)::BIGINT AS "count!" FROM (
            (SELECT EXTRACT(HOUR FROM a.created_at)::INT AS hour, COUNT(*)::BIGINT AS cnt
            FROM user_activity a
            WHERE a.created_at >= NOW() - INTERVAL '24 hours'
            GROUP BY hour)
            UNION ALL
            (SELECT EXTRACT(HOUR FROM m.created_at)::INT AS hour, COUNT(*)::BIGINT AS cnt
            FROM mcp_tool_executions m
            WHERE m.created_at >= NOW() - INTERVAL '24 hours'
            GROUP BY hour)
        ) combined
        GROUP BY hour ORDER BY hour"#,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_stats_snapshot(pool: &PgPool) -> Result<ActivityStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COALESCE(COUNT(*) FILTER (WHERE created_at >= CURRENT_DATE), 0)::BIGINT AS "events_today!",
            COALESCE(COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS "events_this_week!",
            COALESCE(COUNT(*) FILTER (WHERE category = 'marketplace_edit'), 0)::BIGINT AS "total_edits!",
            COALESCE(COUNT(*) FILTER (WHERE category = 'login'), 0)::BIGINT AS "total_logins!"
        FROM user_activity"#,
    )
    .fetch_one(pool)
    .await?;

    let mcp_row = sqlx::query!(
        r#"SELECT
            COUNT(*)::BIGINT AS "total!",
            COALESCE(COUNT(*) FILTER (WHERE status = 'failed'), 0)::BIGINT AS "errors!",
            COALESCE(COUNT(*) FILTER (WHERE created_at >= CURRENT_DATE), 0)::BIGINT AS "today!",
            COALESCE(COUNT(*) FILTER (WHERE created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS "this_week!"
        FROM mcp_tool_executions"#,
    )
    .fetch_one(pool)
    .await;

    let (mcp_total, mcp_errors, mcp_today, mcp_this_week) = match mcp_row {
        Ok(r) => (r.total, r.errors, r.today, r.this_week),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to query mcp_tool_executions for SSE stats");
            (0, 0, 0, 0)
        }
    };

    Ok(ActivityStats {
        events_today: row.events_today + mcp_today,
        events_this_week: row.events_this_week + mcp_this_week,
        total_edits: row.total_edits,
        mcp_tool_calls: mcp_total,
        mcp_errors,
        total_logins: row.total_logins,
    })
}

pub async fn fetch_recent_mcp_errors(
    pool: &Arc<PgPool>,
) -> Result<Vec<super::super::types::RecentMcpError>, sqlx::Error> {
    let rows = sqlx::query_as!(
        McpErrorRow,
        r#"SELECT
            tool_name AS "tool_name!",
            created_at AS "created_at!"
        FROM mcp_tool_executions
        WHERE status = 'failed'
          AND created_at >= NOW() - INTERVAL '24 hours'
        ORDER BY created_at DESC
        LIMIT 5"#,
    )
    .fetch_all(pool.as_ref())
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| super::super::types::RecentMcpError {
            tool_name: r.tool_name,
            created_at: r.created_at,
        })
        .collect())
}

#[derive(Debug, sqlx::FromRow)]
struct McpErrorRow {
    tool_name: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_tool_success_rates(
    pool: &Arc<PgPool>,
) -> Result<Vec<ToolSuccessRate>, sqlx::Error> {
    sqlx::query_as!(
        ToolSuccessRate,
        r#"SELECT
            COALESCE(m.tool_name, 'unknown') AS "tool_name!",
            COUNT(*)::BIGINT AS "total!",
            COUNT(*) FILTER (WHERE m.status = 'success')::BIGINT AS "successes!",
            COUNT(*) FILTER (WHERE m.status = 'failed')::BIGINT AS "failures!",
            (100.0 * COUNT(*) FILTER (WHERE m.status = 'success') / NULLIF(COUNT(*), 0))::FLOAT8 AS "success_pct!"
        FROM mcp_tool_executions m
        GROUP BY m.tool_name
        HAVING COUNT(*) >= 3
        ORDER BY 5 ASC, 2 DESC LIMIT 15"#,
    )
    .fetch_all(pool.as_ref())
    .await
}
