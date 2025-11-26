use anyhow::Result;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum, DbPool, JsonRow};

use super::models::{ConversationSummary, EvaluationStats, RecentConversation};

pub struct ConversationsRepository {
    pool: DbPool,
}

impl ConversationsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub async fn get_conversation_summary(&self, days: i32) -> Result<ConversationSummary> {
        let query = DatabaseQueryEnum::GetConversationSummary.get(self.pool.as_ref());
        let row = self.pool.fetch_one(&query, &[&days]).await?;

        Ok(ConversationSummary {
            total_conversations: row
                .get("total_conversations")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            total_messages: row
                .get("total_messages")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
            avg_messages_per_conversation: row
                .get("avg_messages_per_conversation")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            avg_execution_time_ms: row
                .get("avg_execution_time_ms")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0),
            failed_conversations: row
                .get("failed_conversations")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32,
        })
    }

    pub async fn get_evaluation_stats(&self, days: i32) -> Result<EvaluationStats> {
        let start_date = chrono::Utc::now() - chrono::Duration::days(days.into());
        let end_date = chrono::Utc::now();

        let query = DatabaseQueryEnum::GetEvaluationMetrics.get(self.pool.as_ref());
        let rows = self
            .pool
            .fetch_all(&query, &[&start_date, &end_date])
            .await?;

        let evaluated = rows.len() as i32;
        let avg_quality = rows
            .iter()
            .filter_map(|r| r.get("conversation_quality").and_then(|v| v.as_i64()))
            .sum::<i64>() as f64
            / if evaluated > 0 { evaluated as f64 } else { 1.0 };

        let goal_achievements = rows
            .iter()
            .filter(|r| {
                r.get("goal_achieved")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "yes")
                    .unwrap_or(false)
            })
            .count();

        let goal_achievement_rate = if evaluated > 0 {
            (goal_achievements as f64 / evaluated as f64) * 100.0
        } else {
            0.0
        };

        let avg_satisfaction = rows
            .iter()
            .filter_map(|r| r.get("user_satisfied").and_then(|v| v.as_i64()))
            .sum::<i64>() as f64
            / if evaluated > 0 { evaluated as f64 } else { 1.0 };

        Ok(EvaluationStats {
            evaluated_conversations: evaluated,
            avg_quality_score: avg_quality,
            goal_achievement_rate,
            avg_user_satisfaction: avg_satisfaction,
        })
    }

    pub async fn get_recent_conversations_paginated(
        &self,
        days: i32,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RecentConversation>> {
        let query = DatabaseQueryEnum::GetRecentConversationsPaginated.get(self.pool.as_ref());
        let rows = self
            .pool
            .fetch_all(&query, &[&days, &limit, &offset])
            .await?;

        Ok(rows.iter().map(parse_recent_conversation).collect())
    }

    pub async fn get_conversation_trends(&self) -> Result<Vec<JsonRow>> {
        let query = DatabaseQueryEnum::GetConversationMetricsMultiPeriod.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[]).await?;
        Ok(rows)
    }

    pub async fn get_conversations_by_agent(&self, days: i32) -> Result<Vec<JsonRow>> {
        let query = DatabaseQueryEnum::GetConversationsByAgent.get(self.pool.as_ref());
        let rows = self.pool.fetch_all(&query, &[&days]).await?;
        Ok(rows)
    }
}

fn parse_recent_conversation(r: &JsonRow) -> RecentConversation {
    RecentConversation {
        context_id: r
            .get("context_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        conversation_name: r
            .get("conversation_name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        user_id: r
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous")
            .to_string(),
        user_name: r
            .get("user_name")
            .and_then(|v| v.as_str())
            .unwrap_or("anonymous")
            .to_string(),
        agent_name: r
            .get("agent_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        started_at: r
            .get("started_at")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        started_at_formatted: r
            .get("started_at_formatted")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        duration_seconds: r
            .get("duration_seconds")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0),
        duration_status: r
            .get("duration_status")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        status: r
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        message_count: r.get("message_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        quality_score: r
            .get("quality_score")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        goal_achieved: r
            .get("goal_achieved")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        user_satisfaction: r
            .get("user_satisfaction")
            .and_then(|v| v.as_i64())
            .map(|v| v as i32),
        primary_category: r
            .get("primary_category")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        topics: r
            .get("topics")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        evaluation_summary: r
            .get("evaluation_summary")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
    }
}
