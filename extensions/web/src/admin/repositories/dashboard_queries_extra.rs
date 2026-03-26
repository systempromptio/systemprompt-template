use std::sync::Arc;

use sqlx::PgPool;

use super::super::activity::ActivityTimelineEvent;
use super::super::types::{
    DepartmentActivity, McpAccessSummary, ModelUsage, ProjectActivity, ToolSuccessRate,
};

pub async fn fetch_department_activity(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<DepartmentActivity>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, DepartmentActivity>(
            r"SELECT u.department, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p JOIN users u ON u.id = p.user_id
            WHERE u.department IS NOT NULL AND u.department != ''
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY u.department ORDER BY count DESC",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, DepartmentActivity>(
            r"SELECT u.department, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p JOIN users u ON u.id = p.user_id
            WHERE u.department IS NOT NULL AND u.department != ''
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY u.department ORDER BY count DESC",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_model_usage(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<ModelUsage>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, ModelUsage>(
            r"SELECT COALESCE(p.metadata->>'model', 'unknown') AS model, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.event_type = 'claude_code_SessionStart' AND p.metadata->>'model' IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY model ORDER BY count DESC LIMIT 10",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, ModelUsage>(
            r"SELECT COALESCE(p.metadata->>'model', 'unknown') AS model, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.event_type = 'claude_code_SessionStart' AND p.metadata->>'model' IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY model ORDER BY count DESC LIMIT 10",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_project_activity(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<ProjectActivity>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, ProjectActivity>(
            r"SELECT
                p.metadata->>'project_path' AS project_path,
                REVERSE(SPLIT_PART(REVERSE(p.metadata->>'project_path'), '/', 1)) AS project_name,
                COUNT(*)::BIGINT AS event_count,
                COUNT(DISTINCT p.session_id)::BIGINT AS session_count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.metadata->>'project_path' IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY p.metadata->>'project_path'
            ORDER BY event_count DESC LIMIT 10",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, ProjectActivity>(
            r"SELECT
                p.metadata->>'project_path' AS project_path,
                REVERSE(SPLIT_PART(REVERSE(p.metadata->>'project_path'), '/', 1)) AS project_name,
                COUNT(*)::BIGINT AS event_count,
                COUNT(DISTINCT p.session_id)::BIGINT AS session_count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.metadata->>'project_path' IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY p.metadata->>'project_path'
            ORDER BY event_count DESC LIMIT 10",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_tool_success_rates(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<ToolSuccessRate>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, ToolSuccessRate>(
            r"SELECT
                COALESCE(p.tool_name, 'unknown') AS tool_name,
                COUNT(*)::BIGINT AS total,
                COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse')::BIGINT AS successes,
                COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUseFailure')::BIGINT AS failures,
                (100.0 * COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse') / COUNT(*))::FLOAT8 AS success_pct
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.event_type IN ('claude_code_PostToolUse', 'claude_code_PostToolUseFailure')
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY p.tool_name
            HAVING COUNT(*) >= 5
            ORDER BY success_pct ASC, total DESC LIMIT 15",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, ToolSuccessRate>(
            r"SELECT
                COALESCE(p.tool_name, 'unknown') AS tool_name,
                COUNT(*)::BIGINT AS total,
                COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse')::BIGINT AS successes,
                COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUseFailure')::BIGINT AS failures,
                (100.0 * COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse') / COUNT(*))::FLOAT8 AS success_pct
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.event_type IN ('claude_code_PostToolUse', 'claude_code_PostToolUseFailure')
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY p.tool_name
            HAVING COUNT(*) >= 5
            ORDER BY success_pct ASC, total DESC LIMIT 15",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_mcp_access_events(
    pool: &Arc<PgPool>,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as::<_, ActivityTimelineEvent>(
        r"SELECT a.id, a.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
            a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
        FROM user_activity a
        LEFT JOIN users u ON u.id = a.user_id
        WHERE a.category = 'mcp_access'
        ORDER BY
            CASE WHEN a.action = 'rejected' THEN 0 ELSE 1 END,
            a.created_at DESC
        LIMIT 50",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn fetch_mcp_access_stats(
    pool: &Arc<PgPool>,
) -> Result<Vec<McpAccessSummary>, sqlx::Error> {
    sqlx::query_as::<_, McpAccessSummary>(
        r"SELECT
            COALESCE(entity_name, 'unknown') AS server_name,
            COALESCE(COUNT(*) FILTER (WHERE action = 'authenticated'), 0)::BIGINT AS granted,
            COALESCE(COUNT(*) FILTER (WHERE action = 'rejected'), 0)::BIGINT AS rejected,
            COALESCE(COUNT(*) FILTER (WHERE action = 'used'), 0)::BIGINT AS tool_calls
        FROM user_activity
        WHERE category = 'mcp_access'
        GROUP BY entity_name
        ORDER BY rejected DESC, granted DESC",
    )
    .fetch_all(pool.as_ref())
    .await
}
