use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_identifiers::AiToolCallId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ToolExecutionRequest {
    pub tool_name: String,
    pub mcp_server_name: String,
    pub input: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub context: systemprompt_core_system::RequestContext,
    pub request_method: Option<String>,
    pub request_source: Option<String>,
    pub ai_tool_call_id: Option<AiToolCallId>,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionResult {
    pub output: Option<serde_json::Value>,
    pub output_schema: Option<serde_json::Value>,
    pub status: String,
    pub error_message: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MCPService {
    pub id: Uuid,
    pub name: String,
    pub module: String,
    pub port: i32,
    pub pid: Option<i32>,
    pub status: String,
    pub health: String,
    pub restart_count: i32,
    pub last_health_check: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MCPService {
    pub fn is_running(&self) -> bool {
        self.status == "running"
    }

    pub fn is_healthy(&self) -> bool {
        self.health == "healthy"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolExecution {
    pub mcp_execution_id: String,
    pub tool_name: String,
    pub mcp_server_name: String,
    pub context_id: Option<String>,
    pub ai_tool_call_id: Option<String>,
    pub user_id: String,
    pub status: String,
    pub input: String,
    pub output: Option<String>,
    pub error_message: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolStats {
    pub tool_name: String,
    pub server_name: String,
    pub total_executions: i64,
    pub success_count: i64,
    pub error_count: i64,
    pub avg_duration_ms: Option<i64>,
    pub min_duration_ms: Option<i64>,
    pub max_duration_ms: Option<i64>,
}
