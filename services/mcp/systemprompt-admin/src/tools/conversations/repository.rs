use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{ConversationSummary, EvaluationStats, RecentConversation};

pub struct ConversationsRepository {
    pool: Arc<PgPool>,
}

impl ConversationsRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn get_conversation_summary(&self, days: i32) -> Result<ConversationSummary> {
        let row = sqlx::query!(
            r#"
            SELECT
                COUNT(DISTINCT uc.context_id) as total_conversations,
                COUNT(tm.id) as total_messages,
                COALESCE(AVG(message_counts.msg_count)::float8, 0) as avg_messages_per_conversation,
                COALESCE(AVG(EXTRACT(EPOCH FROM (at.completed_at - at.started_at)) * 1000)::float8, 0) as avg_execution_time_ms,
                COUNT(DISTINCT CASE WHEN at.status = 'failed' THEN uc.context_id END) as failed_conversations
            FROM user_contexts uc
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            LEFT JOIN task_messages tm ON tm.task_id = at.task_id
            LEFT JOIN (
                SELECT at2.context_id, COUNT(tm2.id) as msg_count
                FROM agent_tasks at2
                JOIN task_messages tm2 ON tm2.task_id = at2.task_id
                GROUP BY at2.context_id
            ) message_counts ON message_counts.context_id = uc.context_id
            WHERE uc.created_at >= NOW() - ($1 || ' days')::INTERVAL
            "#,
            days.to_string()
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(ConversationSummary {
            total_conversations: row.total_conversations.unwrap_or(0) as i32,
            total_messages: row.total_messages.unwrap_or(0) as i32,
            avg_messages_per_conversation: row.avg_messages_per_conversation.unwrap_or(0.0),
            avg_execution_time_ms: row.avg_execution_time_ms.unwrap_or(0.0),
            failed_conversations: row.failed_conversations.unwrap_or(0) as i32,
        })
    }

    pub async fn get_evaluation_stats(&self, days: i32) -> Result<EvaluationStats> {
        let start_date = Utc::now() - chrono::Duration::days(days.into());
        let end_date = Utc::now();

        let rows = sqlx::query!(
            r#"
            SELECT
                conversation_quality,
                goal_achieved,
                user_satisfied
            FROM conversation_evaluations
            WHERE analyzed_at >= $1 AND analyzed_at <= $2
            "#,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        let evaluated = rows.len() as i32;
        let avg_quality = rows.iter().map(|r| r.conversation_quality).sum::<i32>() as f64
            / if evaluated > 0 { evaluated as f64 } else { 1.0 };

        let goal_achievements = rows.iter().filter(|r| r.goal_achieved == "yes").count();

        let goal_achievement_rate = if evaluated > 0 {
            (goal_achievements as f64 / evaluated as f64) * 100.0
        } else {
            0.0
        };

        let avg_satisfaction = rows.iter().map(|r| r.user_satisfied).sum::<i32>() as f64
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
        let rows = sqlx::query!(
            r#"
            SELECT
                uc.context_id,
                uc.name as conversation_name,
                uc.user_id,
                COALESCE(u.name, 'anonymous') as user_name,
                COALESCE(at.agent_name, 'unknown') as agent_name,
                uc.created_at::text as started_at,
                TO_CHAR(uc.created_at, 'YYYY-MM-DD HH24:MI') as started_at_formatted,
                uc.updated_at::text as last_updated,
                TO_CHAR(uc.updated_at, 'YYYY-MM-DD HH24:MI') as last_updated_formatted,
                EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at))::float8 as duration_seconds,
                CASE
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 60 THEN 'quick'
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 300 THEN 'normal'
                    ELSE 'long'
                END as duration_status,
                COALESCE(at.status, 'unknown') as status,
                COALESCE((
                    SELECT COUNT(*)::integer
                    FROM task_messages tm
                    JOIN agent_tasks at2 ON tm.task_id = at2.task_id
                    WHERE at2.context_id = uc.context_id
                ), 0) as message_count,
                ce.conversation_quality as "quality_score?",
                ce.goal_achieved as "goal_achieved?",
                ce.user_satisfied as "user_satisfaction?",
                ce.primary_category as "primary_category?",
                ce.topics_discussed as "topics?",
                ce.evaluation_summary as "evaluation_summary?"
            FROM user_contexts uc
            LEFT JOIN users u ON u.id = uc.user_id
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            LEFT JOIN conversation_evaluations ce ON ce.context_id = uc.context_id
            WHERE uc.created_at >= NOW() - ($1 || ' days')::INTERVAL
            ORDER BY uc.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            days.to_string(),
            limit as i64,
            offset as i64
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| RecentConversation {
                context_id: r.context_id,
                conversation_name: Some(r.conversation_name),
                user_id: r.user_id,
                user_name: r.user_name.unwrap_or_else(|| "anonymous".to_string()),
                agent_name: r.agent_name.unwrap_or_else(|| "unknown".to_string()),
                started_at: r.started_at.unwrap_or_default(),
                started_at_formatted: r.started_at_formatted,
                last_updated: r.last_updated.unwrap_or_default(),
                last_updated_formatted: r.last_updated_formatted,
                duration_seconds: r.duration_seconds.unwrap_or(0.0),
                duration_status: r.duration_status,
                status: r.status.unwrap_or_else(|| "unknown".to_string()),
                message_count: r.message_count.unwrap_or(0),
                quality_score: r.quality_score,
                goal_achieved: r.goal_achieved,
                user_satisfaction: r.user_satisfaction,
                primary_category: r.primary_category,
                topics: r.topics,
                evaluation_summary: r.evaluation_summary,
            })
            .collect())
    }

    pub async fn get_conversation_trends(&self) -> Result<Vec<ConversationTrendRow>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '24 hours') as conversations_24h,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days') as conversations_7d,
                COUNT(*) FILTER (WHERE created_at >= NOW() - INTERVAL '30 days') as conversations_30d
            FROM user_contexts
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(vec![ConversationTrendRow {
            conversations_24h: rows.conversations_24h.unwrap_or(0),
            conversations_7d: rows.conversations_7d.unwrap_or(0),
            conversations_30d: rows.conversations_30d.unwrap_or(0),
        }])
    }

    pub async fn get_conversations_by_agent(&self, days: i32) -> Result<Vec<AgentConversationRow>> {
        let rows = sqlx::query!(
            r#"
            SELECT
                COALESCE(at.agent_name, 'unknown') as agent_name,
                COUNT(DISTINCT uc.context_id) as conversation_count
            FROM user_contexts uc
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            WHERE uc.created_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY at.agent_name
            ORDER BY conversation_count DESC
            "#,
            days.to_string()
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| AgentConversationRow {
                agent_name: r.agent_name.unwrap_or_else(|| "unknown".to_string()),
                conversation_count: r.conversation_count.unwrap_or(0),
            })
            .collect())
    }
}

pub struct ConversationTrendRow {
    pub conversations_24h: i64,
    pub conversations_7d: i64,
    pub conversations_30d: i64,
}

pub struct AgentConversationRow {
    pub agent_name: String,
    pub conversation_count: i64,
}
