use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use systemprompt_core_database::{DatabaseProvider, DatabaseQuery, DbPool, ToDbValue};
use systemprompt_identifiers::{AiToolCallId, McpExecutionId};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};
use uuid::Uuid;

const GET_CONTEXT_ID_FROM_EXECUTION: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/get_context_id_from_execution.sql"
));

const UPDATE_CONTEXT_TIMESTAMP: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/update_context_timestamp.sql"
));

const GET_EXECUTION_BY_ID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/get_execution_by_id.sql"
));

const START_EXECUTION: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/start_execution.sql"
));

const COMPLETE_EXECUTION: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/complete_execution.sql"
));

const LOG_EXECUTION_SYNC: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/log_execution_sync.sql"
));

const GET_TOOL_STATS: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/get_tool_stats.sql"
));

const GET_EXECUTION_ID_BY_AI_CALL_ID: DatabaseQuery = DatabaseQuery::new(include_str!(
    "../queries/tool_usage/postgres/get_execution_id_by_ai_call_id.sql"
));

#[derive(Debug, Clone)]
pub struct ToolUsageRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for ToolUsageRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

#[derive(Debug, Clone)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub mcp_server_name: String,
    pub input: JsonValue,
    pub started_at: DateTime<Utc>,
    pub context: systemprompt_core_system::RequestContext,
    pub request_method: Option<String>,
    pub request_source: Option<String>,
    pub ai_tool_call_id: Option<AiToolCallId>,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub output: Option<JsonValue>,
    pub output_schema: Option<JsonValue>,
    pub status: String,
    pub error_message: Option<String>,
    pub completed_at: DateTime<Utc>,
}

impl ToolUsageRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn start_execution(&self, request: &ToolExecutionRequest) -> Result<McpExecutionId> {
        let mcp_execution_id = McpExecutionId::from(Uuid::new_v4().to_string());
        let input_json = serde_json::to_string(&request.input)?;
        let status = "pending";
        let ai_tool_call_id_str = request.ai_tool_call_id.as_ref().map(std::convert::AsRef::as_ref);

        let db: &dyn DatabaseProvider = self.db_pool.as_ref();
        let insert_result = db
            .execute(
                &START_EXECUTION,
                &[
                    &mcp_execution_id.as_ref(),
                    &request.tool_name.as_str(),
                    &request.mcp_server_name.as_str(),
                    &request.started_at,
                    &input_json.as_str(),
                    &status,
                    &request.context.user_id().as_str(),
                    &request.context.session_id().as_str(),
                    &request.context.context_id().as_str(),
                    &request.context.task_id().as_ref().map(|t| t.as_str()),
                    &request.context.trace_id().as_str(),
                    &request.request_method,
                    &request.request_source,
                    &ai_tool_call_id_str,
                ],
            )
            .await;

        match insert_result {
            Ok(_) => Ok(mcp_execution_id),
            Err(e) => {
                let error_msg = e.to_string();
                let is_duplicate_error = (error_msg.contains("UNIQUE constraint failed")
                    || error_msg.contains("duplicate key value violates unique constraint"))
                    && error_msg.contains("ai_tool_call_id");
                if is_duplicate_error {
                    if let Some(ai_tool_call_id) = &request.ai_tool_call_id {
                        if let Some(existing_id) = self
                            .get_execution_id_by_ai_call_id(ai_tool_call_id.as_ref())
                            .await?
                        {
                            return Ok(existing_id);
                        }
                    }
                }

                Err(e)
            },
        }
    }

    pub async fn complete_execution(
        &self,
        mcp_execution_id: &McpExecutionId,
        result: &ToolExecutionResult,
    ) -> Result<()> {
        let output_json = result
            .output
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let output_schema_json = result
            .output_schema
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let db: &dyn DatabaseProvider = self.db_pool.as_ref();
        let _rows_affected = db
            .execute(
                &COMPLETE_EXECUTION,
                &[
                    &result.completed_at,
                    &output_json,
                    &output_schema_json,
                    &result.status.as_str(),
                    &result.error_message,
                    &mcp_execution_id.as_ref(),
                ],
            )
            .await?;

        let context_row = db
            .fetch_optional(
                &GET_CONTEXT_ID_FROM_EXECUTION,
                &[&mcp_execution_id.as_ref()],
            )
            .await?;

        if let Some(row) = context_row {
            if let Some(context_id) = row.get("context_id").and_then(|v| v.as_str()) {
                let ctx_id = context_id.to_string();
                if !ctx_id.is_empty() {
                    db.execute(&UPDATE_CONTEXT_TIMESTAMP, &[&ctx_id.as_str()])
                        .await
                        .ok();
                }
            }
        }

        Ok(())
    }

    pub async fn log_execution_sync(
        &self,
        request: &ToolExecutionRequest,
        result: &ToolExecutionResult,
    ) -> Result<McpExecutionId> {
        let mcp_execution_id = McpExecutionId::from(Uuid::new_v4().to_string());
        let input_json = serde_json::to_string(&request.input)?;
        let output_json = result
            .output
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let output_schema_json = result
            .output_schema
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let execution_time_ms =
            result.completed_at.timestamp_millis() - request.started_at.timestamp_millis();

        let ai_tool_call_id_str = request.ai_tool_call_id.as_ref().map(std::convert::AsRef::as_ref);

        let db: &dyn DatabaseProvider = self.db_pool.as_ref();
        db.execute(
            &LOG_EXECUTION_SYNC,
            &[
                &mcp_execution_id.as_ref(),
                &request.tool_name.as_str(),
                &request.mcp_server_name.as_str(),
                &request.started_at,
                &result.completed_at,
                &execution_time_ms,
                &input_json.as_str(),
                &output_json,
                &output_schema_json,
                &result.status.as_str(),
                &result.error_message,
                &request.context.user_id().as_str(),
                &request.context.session_id().as_str(),
                &request.context.context_id().as_str(),
                &request.context.task_id().as_ref().map(|t| t.as_str()),
                &request.context.trace_id().as_str(),
                &request.request_method,
                &request.request_source,
                &ai_tool_call_id_str,
            ],
        )
        .await?;

        db.execute(
            &UPDATE_CONTEXT_TIMESTAMP,
            &[&request.context.context_id().as_str()],
        )
        .await
        .ok();

        Ok(mcp_execution_id)
    }

    pub async fn get_tool_stats(
        &self,
        tool_name: Option<&str>,
        server_name: Option<&str>,
        days: i32,
    ) -> Result<Vec<ToolStats>> {
        let db: &dyn DatabaseProvider = self.db_pool.as_ref();
        let base_query = GET_TOOL_STATS.postgres();
        let mut query = String::from(base_query);

        let mut params: Vec<Box<dyn ToDbValue>> = Vec::new();
        params.push(Box::new(days));

        let mut param_index = 2;

        if let Some(tn) = tool_name {
            query.push_str(&format!(" AND tool_name = ${param_index}"));
            params.push(Box::new(tn.to_string()));
            param_index += 1;
        }

        if let Some(sn) = server_name {
            query.push_str(&format!(" AND mcp_server_name = ${param_index}"));
            params.push(Box::new(sn.to_string()));
        }

        query.push_str(" GROUP BY tool_name, mcp_server_name ORDER BY call_count DESC");

        let param_refs: Vec<&dyn ToDbValue> = params.iter().map(|p| &**p).collect();

        let rows = db.fetch_all(&query, &param_refs).await?;

        rows.iter()
            .map(ToolStats::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn get_execution_by_id(&self, execution_id: &str) -> Result<ToolExecutionRecord> {
        let db: &dyn DatabaseProvider = self.db_pool.as_ref();
        let row = db.fetch_one(&GET_EXECUTION_BY_ID, &[&execution_id]).await?;

        ToolExecutionRecord::from_json_row(&row)
    }

    pub async fn get_execution_id_by_ai_call_id(
        &self,
        ai_tool_call_id: &str,
    ) -> Result<Option<McpExecutionId>> {
        let db: &dyn DatabaseProvider = self.db_pool.as_ref();

        match db
            .fetch_optional(&GET_EXECUTION_ID_BY_AI_CALL_ID, &[&ai_tool_call_id])
            .await?
        {
            Some(row) => {
                let execution_id = row
                    .get("mcp_execution_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing mcp_execution_id in result"))?;
                Ok(Some(McpExecutionId::from(execution_id.to_string())))
            },
            None => Ok(None),
        }
    }
}

#[derive(Debug)]
pub struct ToolStats {
    pub tool_name: String,
    pub mcp_server_name: String,
    pub call_count: i64,
    pub avg_duration_ms: Option<f64>,
    pub min_duration_ms: Option<i64>,
    pub max_duration_ms: Option<i64>,
    pub success_rate: f64,
    pub unique_sessions: i64,
    pub unique_users: i64,
    pub unique_contexts: i64,
}

impl ToolStats {
    fn from_json_row(row: &systemprompt_core_database::JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let tool_name = row
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing tool_name"))?
            .to_string();

        let mcp_server_name = row
            .get("mcp_server_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing mcp_server_name"))?
            .to_string();

        let call_count = row
            .get("call_count")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing call_count"))?;

        let avg_duration_ms = row.get("avg_duration_ms").and_then(serde_json::Value::as_f64);

        let min_duration_ms = row.get("min_duration_ms").and_then(serde_json::Value::as_i64);

        let max_duration_ms = row.get("max_duration_ms").and_then(serde_json::Value::as_i64);

        let success_rate = row
            .get("success_rate")
            .and_then(serde_json::Value::as_f64)
            .ok_or_else(|| anyhow!("Missing success_rate"))?;

        let unique_sessions = row
            .get("unique_sessions")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_sessions"))?;

        let unique_users = row
            .get("unique_users")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_users"))?;

        let unique_contexts = row
            .get("unique_contexts")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing unique_contexts"))?;

        Ok(Self {
            tool_name,
            mcp_server_name,
            call_count,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            success_rate,
            unique_sessions,
            unique_users,
            unique_contexts,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ToolExecutionRecord {
    pub mcp_execution_id: McpExecutionId,
    pub tool_name: String,
    pub mcp_server_name: String,
    pub input: String,
    pub output: Option<String>,
    pub status: String,
}

impl ToolExecutionRecord {
    fn from_json_row(row: &systemprompt_core_database::JsonRow) -> Result<Self> {
        use anyhow::anyhow;

        let mcp_execution_id = row
            .get("mcp_execution_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing mcp_execution_id"))?;
        let mcp_execution_id = McpExecutionId::from(mcp_execution_id.to_string());

        let tool_name = row
            .get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing tool_name"))?
            .to_string();

        let mcp_server_name = row
            .get("mcp_server_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing mcp_server_name"))?
            .to_string();

        let input = row
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing input"))?
            .to_string();

        let output = row.get("output").and_then(|v| v.as_str()).map(String::from);

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing status"))?
            .to_string();

        Ok(Self {
            mcp_execution_id,
            tool_name,
            mcp_server_name,
            input,
            output,
            status,
        })
    }
}
