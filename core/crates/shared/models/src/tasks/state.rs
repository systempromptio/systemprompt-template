use serde::{Deserialize, Serialize};

/// Database record for task persistence.
/// This represents the database table structure, NOT the A2A protocol Task
/// entity. For A2A protocol Task, see
/// `crates/modules/agent/src/models/a2a/task.rs`.
#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TaskRecord {
    pub uuid: String,
    pub context_id: String,
    pub status: String,
    pub status_timestamp: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct TaskMessage {
    pub id: i64,
    pub task_uuid: String,
    pub message_id: String,
    pub role: String,
    pub sequence_number: i64,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: String,
    pub reference_task_ids: Option<Vec<String>>,
    pub created_at: String,
}
