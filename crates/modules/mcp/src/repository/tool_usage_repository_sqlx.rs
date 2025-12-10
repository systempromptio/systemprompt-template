use anyhow::Result;
use chrono::Utc;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::McpExecutionId;
use uuid::Uuid;

use crate::models::{ToolExecution, ToolExecutionRequest, ToolExecutionResult, ToolStats};

#[derive(Debug)]
pub struct ToolUsageRepository {
    pool: Arc<PgPool>,
}

impl ToolUsageRepository {
    pub fn new(db: DbPool) -> Self {
        let pool = db.pool_arc().expect("Database must be PostgreSQL");
        Self { pool }
    }

    pub async fn start_execution(&self, request: &ToolExecutionRequest) -> Result<McpExecutionId> {
        if let Some(ai_call_id) = &request.ai_tool_call_id {
            if let Some(existing_id) = self.get_by_ai_call_id(ai_call_id.as_str()).await? {
                return Ok(McpExecutionId::from(existing_id));
            }
        }

        let id = Uuid::new_v4().to_string();
        let mcp_execution_id = McpExecutionId::from(id.clone());
        let context_id = request.context.context_id().as_str().to_string();
        let user_id = request.context.user_id().as_str().to_string();
        let ai_tool_call_id = request
            .ai_tool_call_id
            .as_ref()
            .map(|id| id.as_str().to_string());
        let input_str = serde_json::to_string(&request.input)?;

        let task_id = request.context.task_id().map(|id| id.as_str().to_string());
        let session_id = request.context.session_id().as_str().to_string();
        let trace_id_str = request.context.trace_id().as_str();
        let trace_id = if trace_id_str.is_empty() {
            None
        } else {
            Some(trace_id_str.to_string())
        };

        sqlx::query!(
            r#"
            INSERT INTO mcp_tool_executions (
                mcp_execution_id, tool_name, mcp_server_name, context_id, ai_tool_call_id,
                user_id, task_id, session_id, trace_id, status, input, started_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'pending', $10, $11)
            "#,
            id,
            request.tool_name,
            request.mcp_server_name,
            context_id,
            ai_tool_call_id,
            user_id,
            task_id,
            session_id,
            trace_id,
            input_str,
            request.started_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(mcp_execution_id)
    }

    pub async fn complete_execution(
        &self,
        mcp_execution_id: &McpExecutionId,
        result: &ToolExecutionResult,
    ) -> Result<()> {
        let id = mcp_execution_id.as_str();
        let duration_ms = (result.completed_at - Utc::now()).num_milliseconds().abs() as i32;
        let output_str = result
            .output
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        sqlx::query!(
            r#"
            UPDATE mcp_tool_executions
            SET status = $1, output = $2, error_message = $3, execution_time_ms = $4, completed_at = $5
            WHERE mcp_execution_id = $6
            "#,
            result.status,
            output_str,
            result.error_message,
            duration_ms,
            result.completed_at,
            id
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_execution_sync(
        &self,
        request: &ToolExecutionRequest,
        result: &ToolExecutionResult,
    ) -> Result<McpExecutionId> {
        let id = Uuid::new_v4().to_string();
        let mcp_execution_id = McpExecutionId::from(id.clone());
        let status = if result.error_message.is_some() {
            "failed"
        } else {
            "success"
        };
        let context_id = request.context.context_id().as_str().to_string();
        let user_id = request.context.user_id().as_str().to_string();
        let task_id = request.context.task_id().map(|id| id.as_str().to_string());
        let session_id = request.context.session_id().as_str().to_string();
        let trace_id_str = request.context.trace_id().as_str();
        let trace_id = if trace_id_str.is_empty() {
            None
        } else {
            Some(trace_id_str.to_string())
        };
        let duration_ms = (result.completed_at - request.started_at).num_milliseconds() as i32;
        let input_str = serde_json::to_string(&request.input)?;
        let output_str = result
            .output
            .as_ref()
            .and_then(|v| serde_json::to_string(v).ok());

        sqlx::query!(
            r#"
            INSERT INTO mcp_tool_executions (
                mcp_execution_id, tool_name, mcp_server_name, context_id, user_id, task_id,
                session_id, trace_id, status, input, output, error_message, execution_time_ms,
                started_at, completed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            id,
            request.tool_name,
            request.mcp_server_name,
            context_id,
            user_id,
            task_id,
            session_id,
            trace_id,
            status,
            input_str,
            output_str,
            result.error_message,
            duration_ms,
            request.started_at,
            result.completed_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(mcp_execution_id)
    }

    pub async fn get_by_id(&self, id: &str) -> Result<Option<ToolExecution>> {
        let row = sqlx::query!(
            r#"SELECT
                mcp_execution_id as "mcp_execution_id!",
                tool_name as "tool_name!",
                mcp_server_name as "mcp_server_name!",
                context_id,
                ai_tool_call_id,
                user_id as "user_id!",
                status as "status!",
                input as "input!",
                output,
                error_message,
                execution_time_ms,
                started_at as "started_at!",
                completed_at
            FROM mcp_tool_executions
            WHERE mcp_execution_id = $1"#,
            id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(|r| ToolExecution {
            mcp_execution_id: r.mcp_execution_id,
            tool_name: r.tool_name,
            mcp_server_name: r.mcp_server_name,
            context_id: r.context_id,
            ai_tool_call_id: r.ai_tool_call_id,
            user_id: r.user_id,
            status: r.status,
            input: r.input,
            output: r.output,
            error_message: r.error_message,
            execution_time_ms: r.execution_time_ms,
            started_at: r.started_at,
            completed_at: r.completed_at,
        }))
    }

    pub async fn get_by_ai_call_id(&self, ai_tool_call_id: &str) -> Result<Option<String>> {
        let result = sqlx::query_scalar!(
            r#"SELECT mcp_execution_id as "mcp_execution_id!" FROM mcp_tool_executions WHERE ai_tool_call_id = $1"#,
            ai_tool_call_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result)
    }

    pub async fn get_context_id(&self, execution_id: &str) -> Result<Option<String>> {
        let result = sqlx::query_scalar!(
            "SELECT context_id FROM mcp_tool_executions WHERE mcp_execution_id = $1",
            execution_id
        )
        .fetch_optional(&*self.pool)
        .await?;
        Ok(result.flatten())
    }

    pub async fn get_tool_stats(&self, limit: i64) -> Result<Vec<ToolStats>> {
        let rows = sqlx::query!(
            r#"SELECT
                tool_name as "tool_name!",
                mcp_server_name as "server_name!",
                COUNT(*)::bigint as "total_executions!",
                COUNT(*) FILTER (WHERE status = 'success')::bigint as "success_count!",
                COUNT(*) FILTER (WHERE status = 'failed')::bigint as "error_count!",
                AVG(execution_time_ms)::bigint as avg_duration_ms,
                MIN(execution_time_ms)::bigint as min_duration_ms,
                MAX(execution_time_ms)::bigint as max_duration_ms
            FROM mcp_tool_executions
            GROUP BY tool_name, mcp_server_name
            ORDER BY COUNT(*) DESC
            LIMIT $1"#,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| ToolStats {
                tool_name: r.tool_name,
                server_name: r.server_name,
                total_executions: r.total_executions,
                success_count: r.success_count,
                error_count: r.error_count,
                avg_duration_ms: r.avg_duration_ms,
                min_duration_ms: r.min_duration_ms,
                max_duration_ms: r.max_duration_ms,
            })
            .collect())
    }

    pub async fn update_context_timestamp(&self, context_id: Uuid) -> Result<()> {
        let now = Utc::now();
        let context_id_str = context_id.to_string();
        sqlx::query!(
            "UPDATE user_contexts SET updated_at = $1 WHERE context_id = $2",
            now,
            context_id_str
        )
        .execute(&*self.pool)
        .await?;
        Ok(())
    }
}
