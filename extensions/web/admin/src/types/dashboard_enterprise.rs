//! Value types for the governance and enterprise dashboard panels.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{AgentId, UserId};

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
    pub user_id: UserId,
    pub tool_name: String,
    pub agent_id: Option<AgentId>,
    pub decision: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GovernanceDecisionRow {
    pub id: String,
    pub user_id: UserId,
    pub tool_name: String,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, FromRow)]
pub struct WindowedCounts {
    pub decisions: i64,
    pub denied: i64,
    pub secret_blocks: i64,
    pub distinct_actors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TopActor {
    pub user_id: UserId,
    pub display_name: String,
    pub email: Option<String>,
    pub deny_count: i64,
    pub secret_count: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TopPolicy {
    pub policy: String,
    pub tool_name: String,
    pub hits: i64,
    pub distinct_actors: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IncidentGroup {
    pub agent_id: Option<AgentId>,
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub policy: String,
    pub tool_name: String,
    pub attempts: i64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub sample_reason: String,
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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct McpAccessEvent {
    pub server_name: String,
    pub action: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TokenUsageRow {
    pub label: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub event_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct DepartmentQuery {
    pub dept: Option<String>,
}
