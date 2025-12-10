use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use crate::models::analytics::*;

#[derive(Debug)]
pub struct CoreStatsRepository {
    pool: Arc<PgPool>,
}

impl CoreStatsRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn get_platform_overview(&self) -> Result<PlatformOverview> {
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        let last_7d = now - Duration::days(7);
        sqlx::query_as!(
            PlatformOverview,
            r#"
            SELECT
                (SELECT COUNT(*) FROM users WHERE status != 'deleted') as "total_users!",
                (SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE last_activity_at > $1) as "active_users_24h!",
                (SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE last_activity_at > $2) as "active_users_7d!",
                (SELECT COUNT(*) FROM user_sessions) as "total_sessions!",
                (SELECT COUNT(*) FROM user_sessions WHERE ended_at IS NULL) as "active_sessions!",
                (SELECT COUNT(*) FROM user_contexts) as "total_contexts!",
                (SELECT COUNT(*) FROM agent_tasks) as "total_tasks!",
                (SELECT COUNT(*) FROM ai_requests) as "total_ai_requests!"
            "#,
            last_24h,
            last_7d
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_cost_overview(&self) -> Result<CostOverview> {
        let now = Utc::now();
        let last_24h = now - Duration::hours(24);
        let last_7d = now - Duration::days(7);
        let last_30d = now - Duration::days(30);
        sqlx::query_as!(
            CostOverview,
            r#"
            SELECT
                COALESCE(SUM(cost_cents)::float / 100.0, 0.0) as "total_cost!",
                COALESCE(SUM(cost_cents) FILTER (WHERE created_at > $1)::float / 100.0, 0.0) as "cost_24h!",
                COALESCE(SUM(cost_cents) FILTER (WHERE created_at > $2)::float / 100.0, 0.0) as "cost_7d!",
                COALESCE(SUM(cost_cents) FILTER (WHERE created_at > $3)::float / 100.0, 0.0) as "cost_30d!",
                COALESCE(AVG(cost_cents)::float / 100.0, 0.0) as "avg_cost_per_request!"
            FROM ai_requests
            "#,
            last_24h,
            last_7d,
            last_30d
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_activity_trend(&self, days: i32) -> Result<Vec<ActivityTrend>> {
        let cutoff = Utc::now() - Duration::days(days as i64);
        sqlx::query_as!(
            ActivityTrend,
            r#"
            SELECT
                date_trunc('day', gs.date) as "date!",
                COALESCE(s.sessions, 0) as "sessions!",
                COALESCE(c.contexts, 0) as "contexts!",
                COALESCE(t.tasks, 0) as "tasks!",
                COALESCE(a.ai_requests, 0) as "ai_requests!",
                COALESCE(e.tool_executions, 0) as "tool_executions!"
            FROM generate_series($1::timestamptz, NOW(), '1 day') gs(date)
            LEFT JOIN (
                SELECT date_trunc('day', started_at) as day, COUNT(*) as sessions
                FROM user_sessions WHERE started_at > $1
                GROUP BY 1
            ) s ON s.day = date_trunc('day', gs.date)
            LEFT JOIN (
                SELECT date_trunc('day', created_at) as day, COUNT(*) as contexts
                FROM user_contexts WHERE created_at > $1
                GROUP BY 1
            ) c ON c.day = date_trunc('day', gs.date)
            LEFT JOIN (
                SELECT date_trunc('day', created_at) as day, COUNT(*) as tasks
                FROM agent_tasks WHERE created_at > $1
                GROUP BY 1
            ) t ON t.day = date_trunc('day', gs.date)
            LEFT JOIN (
                SELECT date_trunc('day', created_at) as day, COUNT(*) as ai_requests
                FROM ai_requests WHERE created_at > $1
                GROUP BY 1
            ) a ON a.day = date_trunc('day', gs.date)
            LEFT JOIN (
                SELECT date_trunc('day', created_at) as day, COUNT(*) as tool_executions
                FROM mcp_tool_executions WHERE created_at > $1
                GROUP BY 1
            ) e ON e.day = date_trunc('day', gs.date)
            ORDER BY date ASC
            "#,
            cutoff
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_top_users(&self, limit: i64) -> Result<Vec<TopUser>> {
        sqlx::query_as!(
            TopUser,
            r#"
            SELECT
                u.id as user_id,
                u.name as user_name,
                COUNT(DISTINCT s.session_id) as "session_count!",
                COUNT(DISTINCT t.task_id) as "task_count!",
                COUNT(DISTINCT a.request_id) as "ai_request_count!",
                COALESCE(SUM(a.cost_cents)::float / 100.0, 0.0) as "total_cost!"
            FROM users u
            LEFT JOIN user_sessions s ON s.user_id = u.id
            LEFT JOIN agent_tasks t ON t.user_id = u.id
            LEFT JOIN ai_requests a ON a.user_id = u.id
            WHERE u.status NOT IN ('deleted', 'temporary') AND NOT ('anonymous' = ANY(u.roles))
            GROUP BY u.id, u.name
            ORDER BY "ai_request_count!" DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_top_agents(&self, limit: i64) -> Result<Vec<TopAgent>> {
        sqlx::query_as!(
            TopAgent,
            r#"
            SELECT
                agent_name as "agent_name!",
                COUNT(*) as "task_count!",
                COALESCE(
                    COUNT(*) FILTER (WHERE status = 'completed')::float / NULLIF(COUNT(*), 0),
                    0.0
                ) as "success_rate!",
                COALESCE(AVG(EXTRACT(EPOCH FROM (updated_at - created_at)) * 1000)::bigint, 0) as "avg_duration_ms!"
            FROM agent_tasks
            WHERE agent_name IS NOT NULL
            GROUP BY agent_name
            ORDER BY "task_count!" DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_top_tools(&self, limit: i64) -> Result<Vec<TopTool>> {
        sqlx::query_as!(
            TopTool,
            r#"
            SELECT
                tool_name,
                COUNT(*) as "execution_count!",
                COALESCE(
                    COUNT(*) FILTER (WHERE status = 'success')::float / NULLIF(COUNT(*), 0),
                    0.0
                ) as "success_rate!",
                COALESCE(AVG(execution_time_ms), 0)::bigint as "avg_duration_ms!"
            FROM mcp_tool_executions
            GROUP BY tool_name
            ORDER BY "execution_count!" DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }
}
