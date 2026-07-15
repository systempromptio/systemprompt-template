use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{Email, SessionId, SkillId, UserId};

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
    pub recent_activity: Vec<activity::ActivityTimelineEvent>,
    pub activity_summary: Vec<activity::ActivityCategorySummary>,
    pub sessions: Vec<UserSession>,
    pub event_type_breakdown: Vec<EventTypeCount>,
    pub top_tools: Vec<ToolUsageCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub session_id: SessionId,
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

#[derive(Debug, Clone, Copy)]
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
    pub department: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct UsersQuery {
    pub department: Option<String>,
}

/// One row of the `/admin/overview/identity` users table.
///
/// Aggregates `ai_requests` (sessions, contexts, tokens, cost, models, last
/// activity) and `governance_decisions` (denies, secret breaches, scope
/// violations) per user.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserIdentityRow {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub department: String,
    pub is_active: bool,
    pub last_active: Option<DateTime<Utc>>,
    pub requests: i64,
    pub sessions: i64,
    pub contexts: i64,
    pub models: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub denies: i64,
    pub secret_breaches: i64,
    pub scope_violations: i64,
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
