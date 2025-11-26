use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool};

use super::models::{
    AgentToolUsage, AgentUsageRow, ConversationMetrics, DailyTrend, RecentConversation,
    ToolNameUsage, ToolUsageData, ToolUsageRow, TrafficSummary,
};

pub struct DashboardRepository {
    pool: DbPool,
}

impl DashboardRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_conversation_metrics(&self) -> Result<ConversationMetrics> {
        let query = DatabaseQueryEnum::GetConversationMetricsMultiPeriod.get(self.pool.as_ref());
        let row = self.pool.fetch_one(&query, &[]).await?;

        Ok(ConversationMetrics {
            conversations_24h: row
                .get("conversations_24h")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            conversations_7d: row
                .get("conversations_7d")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            conversations_30d: row
                .get("conversations_30d")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            conversations_prev_24h: row
                .get("conversations_prev_24h")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            conversations_prev_7d: row
                .get("conversations_prev_7d")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            conversations_prev_30d: row
                .get("conversations_prev_30d")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        })
    }

    pub async fn get_recent_conversations(&self, limit: i32) -> Result<Vec<RecentConversation>> {
        let query = DatabaseQueryEnum::GetRecentConversations.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&limit]).await?;

        let conversations: Vec<RecentConversation> = rows
            .iter()
            .filter_map(|r| {
                let context_started_str = r.get("context_started_at").and_then(|v| v.as_str())?;
                let task_started_str = r.get("task_started_at").and_then(|v| v.as_str())?;
                let task_completed_str = r.get("task_completed_at").and_then(|v| v.as_str())?;

                let task_started_at = parse_flexible_timestamp(task_started_str)?;
                let task_completed_at = parse_flexible_timestamp(task_completed_str)?;

                let context_id = r.get("context_id").and_then(|v| v.as_str())?.to_string();
                let agent_name = r.get("agent_name").and_then(|v| v.as_str())?.to_string();
                let status = r.get("status").and_then(|v| v.as_str())?.to_string();
                let message_count = r.get("message_count").and_then(|v| v.as_i64())?;

                Some(RecentConversation {
                    context_id,
                    agent_name,
                    started_at: context_started_str.to_string(),
                    task_started_at,
                    task_completed_at,
                    status,
                    message_count,
                })
            })
            .collect();

        Ok(conversations)
    }

    pub async fn get_traffic_summary(&self, days: i32) -> Result<TrafficSummary> {
        let query = DatabaseQueryEnum::GetTrafficSummary.get(self.pool.as_ref());
        let row = self.pool.fetch_one(&query, &[&days]).await?;

        Ok(TrafficSummary {
            total_sessions: row
                .get("total_sessions")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            total_requests: row
                .get("total_requests")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            unique_users: row
                .get("unique_users")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        })
    }

    pub async fn get_conversation_trends(&self, days: i32) -> Result<Vec<DailyTrend>> {
        let query = DatabaseQueryEnum::GetConversationTrends.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;

        Ok(rows
            .iter()
            .map(|r| DailyTrend {
                date: r
                    .get("date")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                conversations: r.get("conversations").and_then(|v| v.as_i64()).unwrap_or(0),
                tool_executions: r
                    .get("tool_executions")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                active_users: r.get("active_users").and_then(|v| v.as_i64()).unwrap_or(0),
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
        let query = format!(
            r#"
            SELECT
                COALESCE(at.agent_name, 'Unknown') as agent_name,
                COUNT(*) as count
            FROM mcp_tool_executions mte
            LEFT JOIN agent_tasks at ON mte.task_id = at.task_id
            WHERE mte.started_at >= NOW() - INTERVAL '{} hours'
            GROUP BY at.agent_name
            ORDER BY count DESC
            LIMIT {}
        "#,
            hours, limit
        );

        let rows = self.pool.fetch_all(&query, &[]).await?;
        let results = rows
            .iter()
            .map(|row| AgentToolUsage {
                agent_name: row
                    .get("agent_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                count: row.get("count").and_then(|v| v.as_i64()).unwrap_or(0),
            })
            .collect();
        Ok(results)
    }

    async fn get_tool_usage_by_tool_name(
        &self,
        hours: i32,
        limit: i32,
    ) -> Result<Vec<ToolNameUsage>> {
        let query = format!(
            r#"
            SELECT
                tool_name,
                COUNT(*) as count
            FROM mcp_tool_executions
            WHERE started_at >= NOW() - INTERVAL '{} hours'
            GROUP BY tool_name
            ORDER BY count DESC
            LIMIT {}
        "#,
            hours, limit
        );

        let rows = self.pool.fetch_all(&query, &[]).await?;
        let results = rows
            .iter()
            .map(|row| ToolNameUsage {
                tool_name: row
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                count: row.get("count").and_then(|v| v.as_i64()).unwrap_or(0),
            })
            .collect();
        Ok(results)
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
