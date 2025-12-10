use crate::error::RepositoryError;
use crate::models::{
    AIRequest, AIRequestMessage, AIRequestToolCall, AiRequestRecord, ProviderUsage, RequestStatus,
    UserAIUsage,
};
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::{SessionId, TraceId, UserId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AIRequestRepository {
    pool: Arc<PgPool>,
}

impl AIRequestRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: &UserId,
        session_id: Option<&SessionId>,
        trace_id: Option<&TraceId>,
        provider: &str,
        model: &str,
        temperature: Option<f64>,
        max_tokens: Option<i32>,
    ) -> Result<AIRequest, RepositoryError> {
        let id = Uuid::new_v4().to_string();
        let request_id = Uuid::new_v4().to_string();
        let session_id_str = session_id.map(|s| s.as_str().to_string());
        let trace_id_str = trace_id.map(|t| t.as_str().to_string());

        sqlx::query_as!(
            AIRequest,
            r#"
            INSERT INTO ai_requests (
                id, request_id, user_id, session_id, trace_id, provider, model,
                temperature, max_tokens, status, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', CURRENT_TIMESTAMP)
            RETURNING id, request_id, user_id, session_id, task_id, context_id, trace_id,
                      provider, model, temperature, top_p, max_tokens, tokens_used,
                      input_tokens, output_tokens, cost_cents, latency_ms, cache_hit,
                      cache_read_tokens, cache_creation_tokens, is_streaming, status,
                      error_message, created_at, updated_at, completed_at
            "#,
            id,
            request_id,
            user_id.as_str(),
            session_id_str,
            trace_id_str,
            provider,
            model,
            temperature,
            max_tokens
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<AIRequest>, RepositoryError> {
        sqlx::query_as!(
            AIRequest,
            r#"
            SELECT id, request_id, user_id, session_id, task_id, context_id, trace_id,
                   provider, model, temperature, top_p, max_tokens, tokens_used,
                   input_tokens, output_tokens, cost_cents, latency_ms, cache_hit,
                   cache_read_tokens, cache_creation_tokens, is_streaming, status,
                   error_message, created_at, updated_at, completed_at
            FROM ai_requests
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn update_completion(
        &self,
        id: &str,
        tokens_used: i32,
        input_tokens: i32,
        output_tokens: i32,
        cost_cents: i32,
        latency_ms: i32,
    ) -> Result<AIRequest, RepositoryError> {
        sqlx::query_as!(
            AIRequest,
            r#"
            UPDATE ai_requests
            SET tokens_used = $1, input_tokens = $2, output_tokens = $3,
                cost_cents = $4, latency_ms = $5, status = 'completed',
                completed_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
            WHERE id = $6
            RETURNING id, request_id, user_id, session_id, task_id, context_id, trace_id,
                      provider, model, temperature, top_p, max_tokens, tokens_used,
                      input_tokens, output_tokens, cost_cents, latency_ms, cache_hit,
                      cache_read_tokens, cache_creation_tokens, is_streaming, status,
                      error_message, created_at, updated_at, completed_at
            "#,
            tokens_used,
            input_tokens,
            output_tokens,
            cost_cents,
            latency_ms,
            id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn update_error(&self, id: &str, error_message: &str) -> Result<(), RepositoryError> {
        sqlx::query!(
            r#"
            UPDATE ai_requests
            SET status = 'error', error_message = $1, completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
            error_message,
            id
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_message(
        &self,
        request_id: &str,
        role: &str,
        content: &str,
        sequence_number: i32,
    ) -> Result<AIRequestMessage, RepositoryError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query_as!(
            AIRequestMessage,
            r#"
            INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id, request_id, role, content, sequence_number, name, tool_call_id, created_at, updated_at
            "#,
            id,
            request_id,
            role,
            content,
            sequence_number
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_messages(
        &self,
        request_id: &str,
    ) -> Result<Vec<AIRequestMessage>, RepositoryError> {
        sqlx::query_as!(
            AIRequestMessage,
            r#"
            SELECT id, request_id, role, content, sequence_number, name, tool_call_id, created_at, updated_at
            FROM ai_request_messages
            WHERE request_id = $1
            ORDER BY sequence_number ASC
            "#,
            request_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_max_sequence(&self, request_id: &str) -> Result<i32, RepositoryError> {
        let result = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(sequence_number), 0) as "max!" FROM ai_request_messages WHERE request_id = $1"#,
            request_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(result)
    }

    pub async fn insert_tool_call(
        &self,
        request_id: &str,
        ai_tool_call_id: &str,
        tool_name: &str,
        tool_input: &str,
        sequence_number: i32,
    ) -> Result<AIRequestToolCall, RepositoryError> {
        let id = Uuid::new_v4().to_string();

        sqlx::query_as!(
            AIRequestToolCall,
            r#"
            INSERT INTO ai_request_tool_calls (id, request_id, ai_tool_call_id, tool_name, tool_input, sequence_number, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id, request_id, tool_name, tool_input, mcp_execution_id, sequence_number, ai_tool_call_id, created_at, updated_at
            "#,
            id,
            request_id,
            ai_tool_call_id,
            tool_name,
            tool_input,
            sequence_number
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_tool_calls(
        &self,
        request_id: &str,
    ) -> Result<Vec<AIRequestToolCall>, RepositoryError> {
        sqlx::query_as!(
            AIRequestToolCall,
            r#"
            SELECT id, request_id, tool_name, tool_input, mcp_execution_id, sequence_number, ai_tool_call_id, created_at, updated_at
            FROM ai_request_tool_calls
            WHERE request_id = $1
            ORDER BY sequence_number ASC
            "#,
            request_id
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_provider_usage(
        &self,
        days: i32,
    ) -> Result<Vec<ProviderUsage>, RepositoryError> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        sqlx::query_as!(
            ProviderUsage,
            r#"
            SELECT
                provider,
                model,
                COUNT(*)::bigint as "request_count!",
                COALESCE(SUM(tokens_used), 0)::bigint as "total_tokens!",
                COALESCE(SUM(cost_cents), 0)::float8 / 100.0 as "total_cost!",
                AVG(latency_ms)::bigint as "avg_latency_ms"
            FROM ai_requests
            WHERE created_at > $1 AND status = 'completed'
            GROUP BY provider, model
            ORDER BY COUNT(*) DESC
            "#,
            cutoff
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_user_usage(&self, user_id: &UserId) -> Result<UserAIUsage, RepositoryError> {
        sqlx::query_as!(
            UserAIUsage,
            r#"
            SELECT
                user_id as "user_id!: UserId",
                COUNT(*)::bigint as "request_count!",
                COALESCE(SUM(tokens_used), 0)::bigint as "total_tokens!",
                COALESCE(SUM(cost_cents), 0)::float8 / 100.0 as "total_cost!",
                AVG(tokens_used)::float8 as "avg_tokens_per_request"
            FROM ai_requests
            WHERE user_id = $1
            GROUP BY user_id
            "#,
            user_id.as_str()
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn get_session_usage(
        &self,
        session_id: &SessionId,
    ) -> Result<UserAIUsage, RepositoryError> {
        sqlx::query_as!(
            UserAIUsage,
            r#"
            SELECT
                user_id as "user_id!: UserId",
                COUNT(*)::bigint as "request_count!",
                COALESCE(SUM(tokens_used), 0)::bigint as "total_tokens!",
                COALESCE(SUM(cost_cents), 0)::float8 / 100.0 as "total_cost!",
                AVG(tokens_used)::float8 as "avg_tokens_per_request"
            FROM ai_requests
            WHERE session_id = $1
            GROUP BY user_id
            "#,
            session_id.as_str()
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(RepositoryError::from)
    }

    pub async fn store(&self, record: &AiRequestRecord) -> Result<String, RepositoryError> {
        let id = Uuid::new_v4().to_string();
        let status = record.status.as_str();

        let completed_at = match record.status {
            RequestStatus::Completed | RequestStatus::Failed => Some(Utc::now()),
            RequestStatus::Pending => None,
        };

        let session_id = record.session_id.as_ref().map(|s| s.as_str().to_string());
        let task_id = record.task_id.as_ref().map(|t| t.as_str().to_string());
        let context_id = record.context_id.as_ref().map(|c| c.as_str().to_string());
        let trace_id = record.trace_id.as_ref().map(|t| t.as_str().to_string());
        let mcp_execution_id = record
            .mcp_execution_id
            .as_ref()
            .map(|m| m.as_str().to_string());
        let error_message = record.error_message.as_deref();

        sqlx::query!(
            r#"
            INSERT INTO ai_requests (
                id, request_id, user_id, session_id, task_id, context_id, trace_id,
                mcp_execution_id, provider, model, tokens_used, input_tokens, output_tokens,
                cache_hit, cache_read_tokens, cache_creation_tokens, is_streaming,
                cost_cents, latency_ms, status, error_message,
                created_at, updated_at, completed_at
            )
            VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                $14, $15, $16, $17, $18, $19, $20, $21,
                CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, $22
            )
            "#,
            id,
            record.request_id,
            record.user_id.as_str(),
            session_id,
            task_id,
            context_id,
            trace_id,
            mcp_execution_id,
            record.provider,
            record.model,
            record.tokens.tokens_used,
            record.tokens.input_tokens,
            record.tokens.output_tokens,
            record.cache.cache_hit,
            record.cache.cache_read_tokens,
            record.cache.cache_creation_tokens,
            record.is_streaming,
            record.cost_cents,
            record.latency_ms,
            status,
            error_message,
            completed_at
        )
        .execute(&*self.pool)
        .await?;
        Ok(id)
    }

    pub async fn add_response_message(
        &self,
        request_id: &str,
        content: &str,
    ) -> Result<(), RepositoryError> {
        let max_seq = self.get_max_sequence(request_id).await?;
        let id = Uuid::new_v4().to_string();

        let seq = max_seq + 1;
        sqlx::query!(
            r#"
            INSERT INTO ai_request_messages (id, request_id, role, content, sequence_number, created_at)
            VALUES ($1, $2, 'assistant', $3, $4, CURRENT_TIMESTAMP)
            "#,
            id,
            request_id,
            content,
            seq
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    pub async fn link_tool_calls_to_recent_executions(
        &self,
        ai_tool_call_ids: &[String],
    ) -> Result<u64, RepositoryError> {
        if ai_tool_call_ids.is_empty() {
            return Ok(0);
        }

        let result = sqlx::query!(
            r#"
            UPDATE ai_request_tool_calls tc
            SET mcp_execution_id = ex.mcp_execution_id
            FROM mcp_tool_executions ex
            WHERE tc.ai_tool_call_id = ex.ai_tool_call_id
              AND tc.ai_tool_call_id = ANY($1)
              AND tc.mcp_execution_id IS NULL
            "#,
            ai_tool_call_ids
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected())
    }
}
