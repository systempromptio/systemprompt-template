use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::super::activity;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardData {
    pub timeline: Vec<activity::ActivityTimelineEvent>,
    pub top_users: Vec<TopUser>,
    pub popular_skills: Vec<SkillCount>,
    pub hourly_activity: Vec<HourlyActivity>,
    pub department_activity: Vec<DepartmentActivity>,
    pub stats: ActivityStats,
    pub model_usage: Vec<ModelUsage>,
    pub event_breakdown: Vec<EventTypeBreakdown>,
    pub usage_timeseries: Vec<TimeSeriesBucket>,
    pub active_users_24h: i64,
    pub avg_session_duration_secs: i64,
    pub project_activity: Vec<ProjectActivity>,
    pub tool_success_rates: Vec<ToolSuccessRate>,
    pub mcp_access_events: Vec<activity::ActivityTimelineEvent>,
    pub mcp_access_stats: Vec<McpAccessSummary>,
    pub governance_events: Vec<GovernanceEvent>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TimeSeriesBucket {
    pub bucket: DateTime<Utc>,
    pub tool_uses: i64,
    pub prompts: i64,
    pub active_users: i64,
    pub sessions: i64,
    pub errors: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DepartmentActivity {
    pub department: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TimelineEvent {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub session_id: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TopUser {
    pub user_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub logins: i64,
    pub prompts: i64,
    pub plugins: i64,
    pub tokens: i64,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SkillCount {
    pub tool_name: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct HourlyActivity {
    pub hour: i32,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ActivityStats {
    pub events_today: i64,
    pub events_this_week: i64,
    pub total_sessions: i64,
    pub error_count: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub subagents_spawned: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
    pub failure_count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ProjectActivity {
    pub project_path: String,
    pub project_name: String,
    pub event_count: i64,
    pub session_count: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ToolSuccessRate {
    pub tool_name: String,
    pub total: i64,
    pub successes: i64,
    pub failures: i64,
    pub success_pct: f64,
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
pub struct LeaderboardEntry {
    pub user_id: String,
    pub display_name: Option<String>,
    pub department: Option<String>,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub events_count: i64,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievement_count: i64,
    pub last_active_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGamificationProfile {
    pub user_id: String,
    pub display_name: Option<String>,
    pub department: Option<String>,
    pub rank_level: i32,
    pub rank_name: String,
    pub total_xp: i64,
    pub xp_to_next_rank: i64,
    pub next_rank_name: Option<String>,
    pub events_count: i64,
    pub unique_skills_count: i32,
    pub unique_plugins_count: i32,
    pub current_streak: i32,
    pub longest_streak: i32,
    pub achievements: Vec<UnlockedAchievement>,
    pub rank_position: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UnlockedAchievement {
    pub achievement_id: String,
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AchievementInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub total_unlocked: i64,
    pub unlock_percentage: f64,
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
pub struct DashboardQuery {
    #[serde(default = "default_range")]
    pub range: String,
}

fn default_range() -> String {
    "7d".to_string()
}

#[derive(Debug, Deserialize)]
pub struct DepartmentQuery {
    pub dept: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub search: Option<String>,
    pub event_type: Option<String>,
    pub plugin_id: Option<String>,
    #[serde(default = "default_events_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EventRow {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub session_id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct EventsResponse {
    pub events: Vec<EventRow>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

fn default_events_limit() -> i64 {
    100
}
