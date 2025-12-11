use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{
    AgentToolUsage, AgentUsageRow, ConversationMetrics, DailyTrend, RecentConversation,
    ToolNameUsage, ToolUsageData, ToolUsageRow, TrafficSummary,
};

pub struct DashboardRepository {
    pool: Arc<PgPool>,
}

impl DashboardRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn get_conversation_metrics(&self) -> Result<ConversationMetrics> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours') as conversations_24h,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days') as conversations_7d,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as conversations_30d,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '48 hours' AND created_at < NOW() - INTERVAL '24 hours') as conversations_prev_24h,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '14 days' AND created_at < NOW() - INTERVAL '7 days') as conversations_prev_7d,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '60 days' AND created_at < NOW() - INTERVAL '30 days') as conversations_prev_30d
            FROM user_contexts
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(ConversationMetrics {
            conversations_24h: row.conversations_24h.unwrap_or(0),
            conversations_7d: row.conversations_7d.unwrap_or(0),
            conversations_30d: row.conversations_30d.unwrap_or(0),
            conversations_prev_24h: row.conversations_prev_24h.unwrap_or(0),
            conversations_prev_7d: row.conversations_prev_7d.unwrap_or(0),
            conversations_prev_30d: row.conversations_prev_30d.unwrap_or(0),
        })
    }

    pub async fn get_recent_conversations(&self, limit: i32) -> Result<Vec<RecentConversation>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                uc.context_id,
                COALESCE(at.agent_name, 'unknown') as agent_name,
                uc.created_at::text as context_started_at,
                at.started_at::text as task_started_at,
                at.completed_at::text as task_completed_at,
                COALESCE(at.status, 'unknown') as status,
                COALESCE((
                    SELECT COUNT(*)
                    FROM task_messages tm
                    JOIN agent_tasks at2 ON tm.task_id = at2.task_id
                    WHERE at2.context_id = uc.context_id
                ), 0) as message_count
            FROM user_contexts uc
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            ORDER BY uc.created_at DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(&*self.pool)
        .await?;

        let conversations: Vec<RecentConversation> = rows
            .into_iter()
            .filter_map(|r| {
                let task_started_str = r.task_started_at?;
                let task_completed_str = r.task_completed_at?;

                let task_started_at = parse_flexible_timestamp(&task_started_str)?;
                let task_completed_at = parse_flexible_timestamp(&task_completed_str)?;

                Some(RecentConversation {
                    context_id: r.context_id,
                    agent_name: r.agent_name.unwrap_or_else(|| "unknown".to_string()),
                    started_at: r.context_started_at.unwrap_or_default(),
                    task_started_at,
                    task_completed_at,
                    status: r.status.unwrap_or_else(|| "unknown".to_string()),
                    message_count: r.message_count.unwrap_or(0),
                })
            })
            .collect();

        Ok(conversations)
    }

    pub async fn get_traffic_summary(&self, days: i32) -> Result<TrafficSummary> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT session_id) as total_sessions,
                SUM(request_count)::bigint as total_requests,
                COUNT(DISTINCT user_id) as unique_users
            FROM user_sessions
            WHERE started_at >= NOW() - ($1 || ' days')::INTERVAL
            "#,
            days.to_string()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(TrafficSummary {
            total_sessions: row.total_sessions.unwrap_or(0) as i32,
            total_requests: row.total_requests.unwrap_or(0) as i32,
            unique_users: row.unique_users.unwrap_or(0) as i32,
        })
    }

    pub async fn get_conversation_trends(&self, days: i32) -> Result<Vec<DailyTrend>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                DATE(uc.created_at)::text as date,
                COUNT(DISTINCT uc.context_id) as conversations,
                COUNT(DISTINCT mte.mcp_execution_id) as tool_executions,
                COUNT(DISTINCT uc.user_id) as active_users
            FROM user_contexts uc
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            LEFT JOIN mcp_tool_executions mte ON mte.task_id = at.task_id
            WHERE uc.created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY DATE(uc.created_at)
            ORDER BY DATE(uc.created_at) DESC
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| DailyTrend {
                date: r.date.unwrap_or_default(),
                conversations: r.conversations.unwrap_or(0),
                tool_executions: r.tool_executions.unwrap_or(0),
                active_users: r.active_users.unwrap_or(0),
            })
            .collect())
    }

    pub async fn get_tool_usage_data(&self) -> Result<ToolUsageData> {
        let agent_usage_24h = self.get_tool_usage_by_agent(24, 10).await?;
        let agent_usage_7d = self.get_tool_usage_by_agent(168, 10).await?;
        let agent_usage_30d = self.get_tool_usage_by_agent(720, 10).await?;

        let mut agent_map: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for agent in &agent_usage_24h {
            agent_map
                .entry(agent.agent_name.clone())
                .or_insert((0, 0, 0))
                .0 = agent.count;
        }
        for agent in &agent_usage_7d {
            agent_map
                .entry(agent.agent_name.clone())
                .or_insert((0, 0, 0))
                .1 = agent.count;
        }
        for agent in &agent_usage_30d {
            agent_map
                .entry(agent.agent_name.clone())
                .or_insert((0, 0, 0))
                .2 = agent.count;
        }

        let mut agent_data: Vec<AgentUsageRow> = agent_map
            .into_iter()
            .filter(|(name, _)| name != "Unknown")
            .map(|(name, (h24, d7, d30))| AgentUsageRow {
                agent_name: name,
                h24,
                d7,
                d30,
            })
            .collect();
        agent_data.sort_by(|a, b| b.h24.cmp(&a.h24));

        let tool_usage_24h = self.get_tool_usage_by_tool_name(24, 10).await?;
        let tool_usage_7d = self.get_tool_usage_by_tool_name(168, 10).await?;
        let tool_usage_30d = self.get_tool_usage_by_tool_name(720, 10).await?;

        let mut tool_map: HashMap<String, (i64, i64, i64)> = HashMap::new();
        for tool in &tool_usage_24h {
            tool_map
                .entry(tool.tool_name.clone())
                .or_insert((0, 0, 0))
                .0 = tool.count;
        }
        for tool in &tool_usage_7d {
            tool_map
                .entry(tool.tool_name.clone())
                .or_insert((0, 0, 0))
                .1 = tool.count;
        }
        for tool in &tool_usage_30d {
            tool_map
                .entry(tool.tool_name.clone())
                .or_insert((0, 0, 0))
                .2 = tool.count;
        }

        let mut tool_data: Vec<ToolUsageRow> = tool_map
            .into_iter()
            .map(|(name, (h24, d7, d30))| ToolUsageRow {
                tool_name: name,
                h24,
                d7,
                d30,
            })
            .collect();
        tool_data.sort_by(|a, b| b.h24.cmp(&a.h24));

        Ok(ToolUsageData {
            agent_data,
            tool_data,
        })
    }

    async fn get_tool_usage_by_agent(&self, hours: i32, limit: i32) -> Result<Vec<AgentToolUsage>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(at.agent_name, 'Unknown') as agent_name,
                COUNT(*) as count
            FROM mcp_tool_executions mte
            LEFT JOIN agent_tasks at ON mte.task_id = at.task_id
            WHERE mte.started_at >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY at.agent_name
            ORDER BY count DESC
            LIMIT $2
            "#,
            hours.to_string(),
            limit as i64
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AgentToolUsage {
                agent_name: r.agent_name.unwrap_or_else(|| "Unknown".to_string()),
                count: r.count.unwrap_or(0),
            })
            .collect())
    }

    async fn get_tool_usage_by_tool_name(
        &self,
        hours: i32,
        limit: i32,
    ) -> Result<Vec<ToolNameUsage>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                tool_name,
                COUNT(*) as count
            FROM mcp_tool_executions
            WHERE started_at >= NOW() - ($1 || ' hours')::INTERVAL
            GROUP BY tool_name
            ORDER BY count DESC
            LIMIT $2
            "#,
            hours.to_string(),
            limit as i64
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ToolNameUsage {
                tool_name: r.tool_name,
                count: r.count.unwrap_or(0),
            })
            .collect())
    }
}

fn parse_flexible_timestamp(timestamp_str: &str) -> Option<DateTime<Utc>> {
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(timestamp_str, "%Y-%m-%d %H:%M:%S%.f") {
        return Some(dt.and_utc());
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(timestamp_str) {
        return Some(dt.with_timezone(&Utc));
    }
    None
}
