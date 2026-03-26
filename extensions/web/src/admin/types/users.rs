use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::super::activity;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserSummary {
    pub user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_active: DateTime<Utc>,
    pub total_events: i64,
    pub last_tool: Option<String>,
    pub custom_skills_count: i64,
    pub preferred_client: Option<String>,
    pub prompts: i64,
    pub sessions: i64,
    pub tokens: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDetail {
    pub user_id: String,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_active: DateTime<Utc>,
    pub total_events: i64,
    pub custom_skills_count: i64,
    pub preferred_client: Option<String>,
    pub created_at: DateTime<Utc>,
    pub skills: Vec<UserSkill>,
    pub recent_activity: Vec<activity::ActivityTimelineEvent>,
    pub activity_summary: Vec<activity::ActivityCategorySummary>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UsageEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSkill {
    pub id: String,
    pub user_id: String,
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub content: String,
    pub enabled: bool,
    pub version: String,
    pub tags: Vec<String>,
    pub base_skill_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentSkill {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<String>,
    pub source_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub skill_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub base_skill_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSkillRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAgent {
    pub id: String,
    pub user_id: String,
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub enabled: bool,
    pub base_agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAgentRequest {
    pub agent_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub base_agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSkillRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserHook {
    pub id: String,
    pub user_id: String,
    pub hook_id: String,
    pub name: String,
    pub description: String,
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
    pub enabled: bool,
    pub base_hook_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserHookRequest {
    pub hook_id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub event: String,
    #[serde(default = "super::super::types::plugins::default_matcher")]
    pub matcher: String,
    pub command: String,
    #[serde(default)]
    pub is_async: bool,
    #[serde(default)]
    pub base_hook_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserHookRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub event: Option<String>,
    pub matcher: Option<String>,
    pub command: Option<String>,
    pub is_async: Option<bool>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub department: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DepartmentStats {
    pub department: String,
    pub user_count: i64,
    pub active_count: i64,
    pub total_events: i64,
    pub active_24h: i64,
    pub active_7d: i64,
    pub total_tokens: i64,
    pub total_prompts: i64,
    pub total_sessions: i64,
    pub sessions_this_week: i64,
    pub sessions_prev_week: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub user_id: String,
    pub display_name: String,
    pub email: String,
    #[serde(default)]
    pub department: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub department: Option<String>,
    pub roles: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillSecret {
    pub id: String,
    pub skill_id: String,
    pub var_name: String,
    pub var_value: String,
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSkillSecretRequest {
    pub var_name: String,
    pub var_value: String,
}
