use anyhow::Result;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;

use super::models::{ConversationSummary, ConversationTrendRow, RecentConversation};

pub struct ConversationsRepository {
    pool: Arc<PgPool>,
}

impl ConversationsRepository {
    pub fn new(db: DbPool) -> Result<Self> {
        let pool = db.pool_arc()?;
        Ok(Self { pool })
    }

    pub async fn get_conversation_summary(&self, interval: &str) -> Result<ConversationSummary> {
        let row = sqlx::query_as!(
            ConversationSummary,
            r#"
            SELECT
                COUNT(DISTINCT uc.context_id)::int4 as "total_conversations!",
                COUNT(tm.id)::int4 as "total_messages!",
                COALESCE(AVG(message_counts.msg_count)::float8, 0) as "avg_messages_per_conversation!",
                COALESCE(AVG(EXTRACT(EPOCH FROM (at.completed_at - at.started_at)) * 1000)::float8, 0) as "avg_execution_time_ms!",
                COUNT(DISTINCT CASE WHEN at.status = 'failed' THEN uc.context_id END)::int4 as "failed_conversations!"
            FROM user_contexts uc
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            LEFT JOIN task_messages tm ON tm.task_id = at.task_id
            LEFT JOIN (
                SELECT at2.context_id, COUNT(tm2.id) as msg_count
                FROM agent_tasks at2
                JOIN task_messages tm2 ON tm2.task_id = at2.task_id
                GROUP BY at2.context_id
            ) message_counts ON message_counts.context_id = uc.context_id
            WHERE uc.updated_at >= NOW() - $1::TEXT::INTERVAL
            "#,
            interval
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_recent_conversations_paginated(
        &self,
        interval: &str,
        limit: i32,
        offset: i32,
        agent_filter: Option<&str>,
    ) -> Result<Vec<RecentConversation>> {
        match agent_filter {
            Some("non-anonymous") => {
                self.get_conversations_non_anonymous(interval, limit, offset)
                    .await
            }
            Some(agent_name) => {
                self.get_conversations_by_agent(interval, limit, offset, agent_name)
                    .await
            }
            None => self.get_conversations_all(interval, limit, offset).await,
        }
    }

    async fn get_conversations_all(
        &self,
        interval: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RecentConversation>> {
        sqlx::query_as!(
            RecentConversation,
            r#"
            SELECT
                uc.context_id as "context_id!",
                uc.name as conversation_name,
                uc.user_id as "user_id!",
                COALESCE(u.name, 'anonymous') as "user_name!",
                COALESCE(at.agent_name, 'unknown') as "agent_name!",
                uc.created_at::text as "started_at!",
                TO_CHAR(uc.created_at, 'YYYY-MM-DD HH24:MI') as started_at_formatted,
                uc.updated_at::text as "last_updated!",
                TO_CHAR(uc.updated_at, 'YYYY-MM-DD HH24:MI') as last_updated_formatted,
                COALESCE(EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at))::float8, 0) as "duration_seconds!",
                CASE
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 60 THEN 'quick'
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 300 THEN 'normal'
                    ELSE 'long'
                END as duration_status,
                COALESCE(at.status, 'unknown') as "status!",
                COALESCE((
                    SELECT COUNT(*)::int4
                    FROM task_messages tm
                    JOIN agent_tasks at2 ON tm.task_id = at2.task_id
                    WHERE at2.context_id = uc.context_id
                ), 0) as "message_count!"
            FROM user_contexts uc
            LEFT JOIN users u ON u.id = uc.user_id
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            WHERE uc.updated_at >= NOW() - $1::TEXT::INTERVAL
            ORDER BY uc.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            interval,
            i64::from(limit),
            i64::from(offset)
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_conversations_non_anonymous(
        &self,
        interval: &str,
        limit: i32,
        offset: i32,
    ) -> Result<Vec<RecentConversation>> {
        sqlx::query_as!(
            RecentConversation,
            r#"
            SELECT
                uc.context_id as "context_id!",
                uc.name as conversation_name,
                uc.user_id as "user_id!",
                COALESCE(u.name, 'anonymous') as "user_name!",
                COALESCE(at.agent_name, 'unknown') as "agent_name!",
                uc.created_at::text as "started_at!",
                TO_CHAR(uc.created_at, 'YYYY-MM-DD HH24:MI') as started_at_formatted,
                uc.updated_at::text as "last_updated!",
                TO_CHAR(uc.updated_at, 'YYYY-MM-DD HH24:MI') as last_updated_formatted,
                COALESCE(EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at))::float8, 0) as "duration_seconds!",
                CASE
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 60 THEN 'quick'
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 300 THEN 'normal'
                    ELSE 'long'
                END as duration_status,
                COALESCE(at.status, 'unknown') as "status!",
                COALESCE((
                    SELECT COUNT(*)::int4
                    FROM task_messages tm
                    JOIN agent_tasks at2 ON tm.task_id = at2.task_id
                    WHERE at2.context_id = uc.context_id
                ), 0) as "message_count!"
            FROM user_contexts uc
            LEFT JOIN users u ON u.id = uc.user_id
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            WHERE uc.updated_at >= NOW() - $1::TEXT::INTERVAL
            AND at.agent_name IS NOT NULL
            AND at.agent_name != 'anonymous'
            AND at.agent_name != 'unknown'
            ORDER BY uc.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            interval,
            i64::from(limit),
            i64::from(offset)
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    async fn get_conversations_by_agent(
        &self,
        interval: &str,
        limit: i32,
        offset: i32,
        agent_name: &str,
    ) -> Result<Vec<RecentConversation>> {
        sqlx::query_as!(
            RecentConversation,
            r#"
            SELECT
                uc.context_id as "context_id!",
                uc.name as conversation_name,
                uc.user_id as "user_id!",
                COALESCE(u.name, 'anonymous') as "user_name!",
                COALESCE(at.agent_name, 'unknown') as "agent_name!",
                uc.created_at::text as "started_at!",
                TO_CHAR(uc.created_at, 'YYYY-MM-DD HH24:MI') as started_at_formatted,
                uc.updated_at::text as "last_updated!",
                TO_CHAR(uc.updated_at, 'YYYY-MM-DD HH24:MI') as last_updated_formatted,
                COALESCE(EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at))::float8, 0) as "duration_seconds!",
                CASE
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 60 THEN 'quick'
                    WHEN EXTRACT(EPOCH FROM (uc.updated_at - uc.created_at)) < 300 THEN 'normal'
                    ELSE 'long'
                END as duration_status,
                COALESCE(at.status, 'unknown') as "status!",
                COALESCE((
                    SELECT COUNT(*)::int4
                    FROM task_messages tm
                    JOIN agent_tasks at2 ON tm.task_id = at2.task_id
                    WHERE at2.context_id = uc.context_id
                ), 0) as "message_count!"
            FROM user_contexts uc
            LEFT JOIN users u ON u.id = uc.user_id
            LEFT JOIN agent_tasks at ON at.context_id = uc.context_id
            WHERE uc.updated_at >= NOW() - $1::TEXT::INTERVAL
            AND at.agent_name = $4
            ORDER BY uc.updated_at DESC
            LIMIT $2 OFFSET $3
            "#,
            interval,
            i64::from(limit),
            i64::from(offset),
            agent_name
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(Into::into)
    }

    pub async fn get_conversation_trends(&self) -> Result<Vec<ConversationTrendRow>> {
        let row = sqlx::query_as!(
            ConversationTrendRow,
            r#"
            SELECT
                COUNT(*) FILTER (WHERE updated_at >= NOW() - INTERVAL '1 hour') as "conversations_1h!",
                COUNT(*) FILTER (WHERE updated_at >= NOW() - INTERVAL '24 hours') as "conversations_24h!",
                COUNT(*) FILTER (WHERE updated_at >= NOW() - INTERVAL '7 days') as "conversations_7d!",
                COUNT(*) FILTER (WHERE updated_at >= NOW() - INTERVAL '30 days') as "conversations_30d!"
            FROM user_contexts
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(vec![row])
    }
}
