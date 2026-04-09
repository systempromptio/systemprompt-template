use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct ActivityFeedEvent {
    pub id: String,
    pub session_id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub description: Option<String>,
    pub prompt_preview: Option<String>,
    pub cwd: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecentSession {
    pub session_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_events: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub subagent_spawns: i64,
    pub status: String,
    pub updated_at: DateTime<Utc>,
    pub client_source: String,
    pub permission_mode: String,
    pub user_prompts: i32,
    pub automated_actions: i32,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSessionStatusRequest {
    pub session_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchUpdateSessionStatusRequest {
    pub session_ids: Vec<String>,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct AnalyseSessionRequest {
    pub session_id: String,
}

#[derive(Debug, Serialize, Default, Clone, Copy)]
pub struct TodayStats {
    pub sessions_started: i64,
    pub total_prompts: i64,
    pub total_tool_calls: i64,
    pub total_errors: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct RecentTask {
    pub task_subject: Option<String>,
    pub task_description: Option<String>,
    pub created_at: DateTime<Utc>,
}
