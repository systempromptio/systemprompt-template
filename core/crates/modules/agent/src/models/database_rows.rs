use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskRow {
    pub task_id: String,
    pub context_id: String,
    pub status: String,
    pub status_timestamp: Option<DateTime<Utc>>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub agent_name: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TaskMessage {
    pub id: i32,
    pub task_id: String,
    pub message_id: String,
    pub client_message_id: Option<String>,
    pub role: String,
    pub context_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub sequence_number: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
    pub reference_task_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessagePart {
    pub id: i32,
    pub message_id: String,
    pub task_id: String,
    pub part_kind: String,
    pub sequence_number: i32,
    pub text_content: Option<String>,
    pub file_name: Option<String>,
    pub file_mime_type: Option<String>,
    pub file_uri: Option<String>,
    pub file_bytes: Option<String>,
    pub data_content: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillRow {
    pub skill_id: String,
    pub file_path: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub enabled: bool,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub source_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtifactRow {
    pub artifact_id: String,
    pub task_id: String,
    pub context_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub artifact_type: String,
    pub source: Option<String>,
    pub tool_name: Option<String>,
    pub mcp_execution_id: Option<String>,
    pub fingerprint: Option<String>,
    pub skill_id: Option<String>,
    pub skill_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub artifact_created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ArtifactPartRow {
    pub id: i32,
    pub artifact_id: String,
    pub context_id: String,
    pub part_kind: String,
    pub sequence_number: i32,
    pub text_content: Option<String>,
    pub file_name: Option<String>,
    pub file_mime_type: Option<String>,
    pub file_uri: Option<String>,
    pub file_bytes: Option<String>,
    pub data_content: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutionStepBatchRow {
    pub step_id: String,
    pub task_id: String,
    pub status: String,
    pub content: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PushNotificationConfigRow {
    pub config_id: String,
    pub task_id: String,
    pub url: String,
    pub token: Option<String>,
    pub authentication_info: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}
