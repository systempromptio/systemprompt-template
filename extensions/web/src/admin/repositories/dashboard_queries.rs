use std::sync::Arc;

use sqlx::PgPool;

use super::super::activity;
use super::super::types::{HourlyActivity, SkillCount, TopUser};

pub use super::dashboard_queries_extra::{
    fetch_department_activity, fetch_model_usage, fetch_project_activity,
    fetch_tool_success_rates,
};

pub async fn fetch_timeline(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<activity::ActivityTimelineEvent>, sqlx::Error> {
    activity::queries::fetch_timeline(pool, department).await
}

pub async fn fetch_top_users(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<TopUser>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, TopUser>(
            r"SELECT p.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS display_name,
                u.email,
                COALESCE(lc.logins, 0)::BIGINT AS logins,
                COUNT(CASE WHEN p.event_type = 'claude_code_UserPromptSubmit' THEN 1 END)::BIGINT AS prompts,
                COUNT(DISTINCT p.plugin_id)::BIGINT AS plugins,
                COALESCE(tt.tokens, 0)::BIGINT AS tokens,
                MAX(p.created_at) AS last_active
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS logins
                FROM user_activity WHERE category = 'login'
                GROUP BY user_id
            ) lc ON lc.user_id = p.user_id
            LEFT JOIN (
                SELECT user_id, (SUM(total_input_tokens) + SUM(total_output_tokens))::BIGINT AS tokens
                FROM plugin_usage_daily
                GROUP BY user_id
            ) tt ON tt.user_id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY p.user_id, u.display_name, u.full_name, u.name, u.email, lc.logins, tt.tokens
            ORDER BY prompts DESC, logins DESC LIMIT 10",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, TopUser>(
            r"SELECT p.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS display_name,
                u.email,
                COALESCE(lc.logins, 0)::BIGINT AS logins,
                COUNT(CASE WHEN p.event_type = 'claude_code_UserPromptSubmit' THEN 1 END)::BIGINT AS prompts,
                COUNT(DISTINCT p.plugin_id)::BIGINT AS plugins,
                COALESCE(tt.tokens, 0)::BIGINT AS tokens,
                MAX(p.created_at) AS last_active
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS logins
                FROM user_activity WHERE category = 'login'
                GROUP BY user_id
            ) lc ON lc.user_id = p.user_id
            LEFT JOIN (
                SELECT user_id, (SUM(total_input_tokens) + SUM(total_output_tokens))::BIGINT AS tokens
                FROM plugin_usage_daily
                GROUP BY user_id
            ) tt ON tt.user_id = p.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY p.user_id, u.display_name, u.full_name, u.name, u.email, lc.logins, tt.tokens
            ORDER BY prompts DESC, logins DESC LIMIT 10",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_popular_skills(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<SkillCount>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, SkillCount>(
            r"SELECT COALESCE(p.tool_name, 'unknown') AS tool_name, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.tool_name IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY p.tool_name ORDER BY count DESC LIMIT 10",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, SkillCount>(
            r"SELECT COALESCE(p.tool_name, 'unknown') AS tool_name, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.tool_name IS NOT NULL
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY p.tool_name ORDER BY count DESC LIMIT 10",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_hourly_activity(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<HourlyActivity>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, HourlyActivity>(
            r"SELECT EXTRACT(HOUR FROM p.created_at)::INT AS hour, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '24 hours'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            GROUP BY hour ORDER BY hour",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, HourlyActivity>(
            r"SELECT EXTRACT(HOUR FROM p.created_at)::INT AS hour, COUNT(*)::BIGINT AS count
            FROM plugin_usage_events p
            JOIN users u ON u.id = p.user_id
            WHERE p.created_at >= NOW() - INTERVAL '24 hours'
              AND NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY hour ORDER BY hour",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}
