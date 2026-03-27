use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{AgentId, CategoryId, Email, SkillId, SourceId, UserId};

use super::super::activity;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserSummary {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub last_active: DateTime<Utc>,
    pub total_events: i64,
    pub last_tool: Option<String>,
    pub custom_skills_count: i64,
    pub preferred_client: Option<String>,
    pub prompts: i64,
    pub sessions: i64,
    pub bytes: i64,
    pub logins: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDetail {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<Email>,
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
    pub sessions: Vec<UserSession>,
    pub event_type_breakdown: Vec<EventTypeCount>,
    pub top_tools: Vec<ToolUsageCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub session_id: String,
    pub started_at: Option<DateTime<Utc>>,
    pub total_events: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub errors: i64,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub subagent_spawns: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EventTypeCount {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ToolUsageCount {
    pub tool_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UsageEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSkill {
    pub id: String,
    pub user_id: UserId,
    pub skill_id: SkillId,
    pub name: String,
    pub description: String,
    pub content: String,
    pub enabled: bool,
    pub version: String,
    pub tags: Vec<String>,
    pub base_skill_id: Option<SkillId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentSkill {
    pub skill_id: SkillId,
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub tags: Option<Vec<String>>,
    pub category_id: Option<CategoryId>,
    pub source_id: SourceId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSkillRequest {
    pub skill_id: SkillId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub base_skill_id: Option<SkillId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAgent {
    pub id: String,
    pub user_id: UserId,
    pub agent_id: AgentId,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub enabled: bool,
    pub base_agent_id: Option<AgentId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserAgentRequest {
    pub agent_id: AgentId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub base_agent_id: Option<AgentId>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserSkillRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub system_prompt: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UserBasicInfo {
    pub display_name: String,
    pub email: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CookieSession {
    pub user_id: UserId,
    pub username: String,
    pub email: Email,
}

#[derive(Debug, Clone)]
pub struct UserPluginCounts {
    pub plugins: usize,
    pub skills: usize,
    pub agents: usize,
    pub mcp_servers: usize,
}

#[derive(Debug, Clone)]
pub struct ContentBytes {
    pub input: i64,
    pub output: i64,
}

#[derive(Debug, Clone)]
pub struct DetectedEntity {
    pub entity_type: &'static str,
    pub entity_name: String,
}

#[derive(Debug, Clone)]
pub struct JwtIdentity {
    pub user_id: UserId,
    pub plugin_id: String,
}

#[derive(Debug, Clone)]
pub struct UserTier {
    pub plan_name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub user_id: UserId,
    pub display_name: String,
    pub email: Email,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SkillSecret {
    pub id: String,
    pub skill_id: SkillId,
    pub var_name: String,
    pub var_value: String,
    pub is_secret: bool,
}

#[derive(Debug, Deserialize)]
pub struct UpsertSkillSecretRequest {
    pub var_name: String,
    pub var_value: String,
}

// --- Enterprise / foodles-specific types ---

#[derive(Debug, Deserialize)]
pub struct UpdateSkillRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
    pub tags: Option<Vec<String>>,
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
