use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;


#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DepartmentActivity {
    pub department: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProjectActivity {
    pub project_path: String,
    pub project_name: String,
    pub event_count: i64,
    pub session_count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct McpAccessSummary {
    pub server_name: String,
    pub granted: i64,
    pub rejected: i64,
    pub tool_calls: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GovernanceEvent {
    pub id: String,
    pub user_id: String,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub decision: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GovernanceDecisionRow {
    pub id: String,
    pub user_id: String,
    pub tool_name: String,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ModelUsage {
    pub model: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventTypeBreakdown {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DepartmentScore {
    pub department: String,
    pub total_xp: i64,
    pub avg_xp: f64,
    pub user_count: i64,
    pub top_user_name: Option<String>,
    pub top_user_xp: i64,
}

#[derive(Debug, Deserialize)]
pub struct DepartmentQuery {
    pub dept: Option<String>,
}
